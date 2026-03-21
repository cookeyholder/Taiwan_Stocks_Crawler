import csv
import datetime
import glob
import os
import random
import sqlite3
import time

import pandas as pd
import requests
from fake_useragent import UserAgent

from you import Line


def _sleep_retry():
    print("休息 5 到 10 秒鐘再抓")
    time.sleep(random.randint(5, 10))


def _normalize_quote_row(data):
    normalized = [i.replace('"', "") for i in data]
    normalized = [i.replace(",", "") for i in normalized]
    normalized = [i.replace("---", "0") for i in normalized]
    normalized = [i.replace("--", "0") for i in normalized]
    return [i.strip() for i in normalized]


def is_warrant(code):
    """
    從代號判斷是不是權證
    上市權證編碼規則請看 https://www.twse.com.tw/zh/page/products/securities/warrant/faq.html
    上櫃權證編碼規則請看 https://www.tpex.org.tw/storage/warrant/%E6%8A%95%E8%B3%87%E4%BA%BA/%E6%8A%95%E8%B3%87%E6%AC%8A%E8%AD%89QA_10510.pdf

    上市權證代碼於 2010-05-14 前為五位數，從 03001 到 08999；2010-05-17 後為六位編碼，為現行規則。
    """
    if len(code) == 6 and code[-1] in ["P", "F", "Q", "C", "B", "X", "Y"]:
        if int(code[:-1]) >= 3001 and int(code[:-1]) <= 8999:
            return True
        elif int(code[:-1]) >= 70000 and int(code[:-1]) <= 73999:
            return True
        else:
            return False
    # code.isdigit() 用來檢查 code 是不是全由數字組成
    elif len(code) == 6 and code.isdigit() and int(code) >= 30001 and int(code) <= 89999:
        return True
    elif len(code) == 6 and code.isdigit() and int(code) >= 700000 and int(code) <= 739999:
        return True
    elif len(code) == 5 and code.isdigit() and int(code) >= 3001 and int(code) <= 8999:
        return True
    elif len(code) == 5 and code.isdigit() and int(code) >= 70000 and int(code) <= 73999:
        return True
    else:
        return False


def Taiwan_date(date):
    """
    將 ISO 格式的日期轉成民國年/月/日
    """
    d = date.split("-")
    year = str(int(d[0]) - 1911)
    month = d[1]
    day = d[2]

    return "/".join([year, month, day])


def find_twse_start_row(lines):
    """
    找出上市交易資料以「證券代號」開頭的列號
    """
    for i in range(len(lines)):
        if [x.replace('"', "").strip() for x in lines[i].split('","')][0] == "證券代號":
            break
    return i


def find_otc_start_row(lines):
    """
    找出上櫃交易資料以「代號」開頭的列號
    """
    start_row = []
    for i in range(len(lines)):
        if [x.replace('"', "").strip() for x in lines[i].split(",")][0] == "代號":
            start_row.append(i)
    return start_row


def twse_crawler(trade_date, output_to_file=True):
    """
    爬取上市公司的每日收盤行情
    """
    # proxy_ip = None
    trade_date = trade_date.strftime("%Y-%m-%d")

    while True:
        # 取得 proxy 的 ip
        # if proxy_ip is None:
        # proxy_ip = youProxies.getProxy()
        print(f"    正在抓取 {trade_date} 的上市每日收盤行情……")

        try:
            twse_url = (
                "http://www.twse.com.tw/exchangeReport/MI_INDEX?response=csv&date="
                + trade_date.replace("-", "")
                + "&type=ALL"
            )

            my_header = {"user-agent": UserAgent().random}  # , 'Connection': 'close'}
            r = requests.get(
                twse_url,
                headers=my_header,
                timeout=5,
                # proxies={'https':f'https://{proxy_ip}'}
            )

            # 如果 requests 沒有成功連線，則將 proxy_ip 設為 None 後再重跑迴圈
            if r.status_code != requests.codes.ok:
                print("沒有成功連線，重設 proxy")
                print("休息 5 到 10 秒鐘再抓")
                time.sleep(random.randint(5, 10))
                # proxy_ip = None
                continue

            # 如果成功連線就會繼續以下的程式碼
            lines = r.text.split("\n")
            lines = lines[find_twse_start_row(lines) : -1]
            twse_stocks = [
                [i.strip() for i in lines.pop(0).replace("=", "").replace('"', "").split(",")]
            ]

            for line in lines:
                data = line.split('","')
                data[0] = data[0].replace("=", "")
                data[0] = data[0].replace('"', "")
                # 如果不是權證
                if not is_warrant(data[0]):
                    twse_stocks.append(_normalize_quote_row(data))

            # 把 twse_stocks 轉成 DataFrame
            twse_stocks = pd.DataFrame(twse_stocks[1:], columns=twse_stocks[0][:-1])
            # twse_stocks = twse_stocks.apply(pd.to_numeric, errors='ignore')
            # 如果 output_to_file 參數是 True 表示要輸出成 CSV 檔
            if output_to_file is True:
                twse_stocks.to_csv(
                    os.path.join("每日收盤行情", f"{trade_date}上市公司股票成交資訊.csv"),
                    encoding="utf-8",
                    index=False,
                )
            twse_stocks["Date"] = trade_date
            twse_stocks = twse_stocks[
                [
                    "Date",
                    "證券代號",
                    "證券名稱",
                    "開盤價",
                    "最高價",
                    "最低價",
                    "收盤價",
                    "成交股數",
                    "成交金額",
                ]
            ]
            twse_stocks.columns = [
                "Date",
                "Symbol",
                "Name",
                "Open",
                "High",
                "Low",
                "Close",
                "Volume",
                "Value",
            ]
            break

        except requests.exceptions.RequestException as e:
            print(type(e).__name__)
            _sleep_retry()
            continue

        except Exception as e:
            print(e)
            _sleep_retry()
            continue
    print(f"        抓取 {trade_date} 的上市每日收盤行情結束")
    return twse_stocks


def otc_crawler(trade_date, output_to_file=True):
    """
    爬取上櫃公司的每日收盤行情
    上櫃的交易明細在 2020-04-30 之後改過欄位數量，所以要分成 2020-04-29 之前和 2020-04-30 之後。
    """
    # proxy_ip = None
    trade_date = trade_date.strftime("%Y-%m-%d")
    print(f"    正在抓取 {trade_date} 的上櫃每日收盤行情……")

    while True:
        # if proxy_ip is None:
        # proxy_ip = youProxies.getProxy()

        try:
            otc_url = (
                "https://www.tpex.org.tw/web/stock/aftertrading/daily_close_quotes/stk_quote_download.php?l=zh-tw&d="
                + Taiwan_date(trade_date)
                + "&s=0,asc,0"
            )

            # otc_url = 'https://www.tpex.org.tw/web/stock/aftertrading/otc_quotes_no1430/stk_wn1430_result.php?l=zh-tw&o=csv&d=' + Taiwan_date(trade_date) + '&se=EW&s=0,asc,0'
            my_header = {"user-agent": UserAgent().random}  # , 'Connection': 'close'}
            r = requests.get(
                otc_url,
                headers=my_header,
                timeout=5,
                # proxies={'https':f'https://{proxy_ip}'}
            )

            if r.status_code != requests.codes.ok:
                print("沒有成功連線，重設 proxy")
                print("休息 5 到 10 秒鐘再抓")
                time.sleep(random.randint(5, 10))
                # proxy_ip = None
                continue

            r.encoding = "big5-hkscs"
            # 上櫃原始的交易明細有空行和管理股票，將要擷取的部分拿出來
            lines = r.text.split("\n")[:-6]
            # 如果沒有以「代號」開頭的列，表示該天沒有交易。
            if len(find_otc_start_row(lines)) != 0:
                a, b = find_otc_start_row(lines)
                lines = lines[a : b - 2] + lines[b + 1 :]

                otc_stocks = [
                    [i.strip() for i in lines.pop(0).replace("=", "").replace('"', "").split(",")]
                ]
                for line in lines:
                    data = line.split('","')
                    data[0] = data[0].replace("=", "")
                    data[0] = data[0].replace('"', "")
                    # 如果不是權證
                    if not is_warrant(data[0]):
                        otc_stocks.append(_normalize_quote_row(data))

                otc_stocks = pd.DataFrame(otc_stocks[1:], columns=otc_stocks[0])
                # otc_stocks = otc_stocks.apply(pd.to_numeric, errors='ignore')
                if output_to_file is True:
                    otc_stocks.to_csv(
                        os.path.join("每日收盤行情", f"{trade_date}上櫃公司股票成交資訊.csv"),
                        encoding="utf-8",
                        index=False,
                    )
                otc_stocks["Date"] = trade_date
                otc_stocks = otc_stocks[
                    [
                        "Date",
                        "代號",
                        "名稱",
                        "開盤",
                        "最高",
                        "最低",
                        "收盤",
                        "成交股數",
                        "成交金額(元)",
                    ]
                ]
                otc_stocks.columns = [
                    "Date",
                    "Symbol",
                    "Name",
                    "Open",
                    "High",
                    "Low",
                    "Close",
                    "Volume",
                    "Value",
                ]
                break
            else:
                otc_stocks = False
                return otc_stocks
                break

        except requests.exceptions.RequestException as e:
            print(type(e).__name__)
            _sleep_retry()
            continue

        except Exception as e:
            print(e)
            _sleep_retry()
            continue
    print(f"        抓取 {trade_date} 的上櫃每日收盤行情結束")
    return otc_stocks


def write_to_db(df, db_name="stocks.sqlite", table_name="OHLCV"):
    """
    將 DataFrame 輸出到資料庫，預設的資料庫名稱為 stocks.sqlite，資料表名稱為 OHLCV。
    db_name：資料庫名稱
    table_name：資料表名稱
    """
    try:
        conn = sqlite3.connect(db_name)
        df.to_sql(table_name, conn, if_exists="append", index=False)
        conn.commit()
        conn.close()
    except Exception as e:
        print(e)
        print(f"{df.iloc[0, 0]} 資料寫入資料庫發生錯誤！")


def main(end_date=datetime.date.today().strftime("%Y-%m-%d")):
    start_time = time.time()
    # 計算已有資料的最後一天
    fn_list = glob.glob(os.path.join("每日收盤行情", "*開高低收量額.csv"))
    start_date = fn_list[-1][:10]
    start_date = datetime.datetime.strptime(start_date, "%Y-%m-%d") + datetime.timedelta(days=1)
    # 將已有資料的最後一天的日期加 1 天
    start_date = start_date.strftime("%Y-%m-%d")

    # 產生所有從已有資料的最後一天到今天的所有日期
    date_list = pd.date_range(start_date, end_date)
    days = 0
    for trade_date in date_list:
        print(f"正在抓取 {trade_date.strftime('%Y-%m-%d')} 的資料...")
        df_otc = otc_crawler(trade_date)
        if df_otc is not False:
            df_twse = twse_crawler(trade_date)
            trade_date = trade_date.strftime("%Y-%m-%d")
            df = pd.concat([df_twse, df_otc])
            df["Symbol"] = df["Symbol"].astype("str")
            df = df.sort_values(by=["Symbol"]).reset_index(drop=True)
            df.to_csv(
                f"./每日收盤行情/{trade_date}臺股開高低收量額.csv", encoding="utf8", index=False
            )
            print(f"        抓取 {trade_date} 的資料結束")
        else:
            print(f"        {trade_date.strftime('%Y-%m-%d')} 沒有交易")
            print(f"        抓取 {trade_date.strftime('%Y-%m-%d')} 的資料結束")
            continue

        write_to_db(df)
        # 休息 5 到 10 秒鐘再抓
        time.sleep(random.randint(5, 10))
        days = days + 1

    msg = f"下載每日收盤行情完成，總共抓取了 {days} 天的交易資料，共用了 {time.time() - start_time} 秒。"
    if Line.Vanessa(msg) != 200:
        msg = "每日收盤行情下載失敗"
        Line.Vanessa(msg)

    print("全部結束。")
    print(f"總共抓取了 {days} 天的交易資料。")
    print(f"總共花費 {time.time() - start_time} 秒。")


def ownership_distribution():
    url = "https://smart.tdcc.com.tw/opendata/getOD.ashx?id=1-5"
    my_header = {"user-agent": UserAgent().random}
    while True:
        try:
            r = requests.get(url, headers=my_header)
            r.encoding = "utf-8-sig"

            ownership = []
            lines = r.text.split("\n")
            for line in lines:
                tmp = [x.strip() for x in line.split(",")]
                ownership.append(tmp)

            date = ownership[1][0]
            date = date[:4] + "-" + date[4:6] + "-" + date[-2:]
            filename = f"集保戶股權分散表{os.sep}" + date + "集保戶股權分散表.csv"
            with open(filename, "w", newline="", encoding="utf-8-sig") as csvfile:
                writer = csv.writer(csvfile)
                writer.writerows(ownership)

            # 發送 Line 訊息通知己下載完成
            msg = f"{date} 所有股票的股權分散表已經下載完成。"
            Line.Vanessa(msg)
            break

        except Exception as e:
            print("Exception was occured.")
            print(e)
            continue


def get_query_dates():
    """
    取得股權分散表選單中的所有日期
    """
    do_url = "https://www.tdcc.com.tw/smWeb/QryStockAjax.do"
    datas = {"REQ_OPR": "qrySelScaDates"}
    my_header = {"user-agent": UserAgent().random}
    r = requests.post(do_url, headers=my_header, data=datas)
    dates = r.text.replace('["', "").replace('"]', "").split('","')
    return dates


def get_stock_ownership(stock_id):
    """
    取得臺灣集中保管結算所的個股股權分散表
    """
    query_dates = get_query_dates()
    for i, date in enumerate(query_dates):
        print(f"{i:>02d}.", "Crawling date is", date)
        while True:
            try:
                url = "https://www.tdcc.com.tw/smWeb/QryStockAjax.do"
                payload = {
                    "scaDates": date,
                    "scaDate": date,
                    "SqlMethod": "StockNo",
                    "StockNo": stock_id,
                    "radioStockNo": stock_id,
                    "StockName": "",
                    "REQ_OPR": "SELECT",
                    "clkStockNo": stock_id,
                    "clkStockName": "",
                }

                my_header = {"user-agent": UserAgent().random}
                r = requests.post(url, data=payload, headers=my_header)
                r.encoding = "big5-hkscs"
                stock_name = pd.read_html(r.text)[5].iloc[0, 0].split("證券名稱：")[1]
                df = pd.read_html(r.text)[6]
                columns = df.iloc[0, :].to_list()
                df.columns = columns
                df = df.drop([0])
                if df.iloc[0, 0] != "無此資料":
                    filename = (
                        f"集保戶股權分散表{os.sep}{stock_id}({stock_name}){date}股權分散表.csv"
                    )
                    if os.path.isfile(filename):
                        print("    File was existed.")
                    else:
                        df.to_csv(filename, encoding="utf-8", index=False)
                        print("   Success, Sleep for 1 seconds.")
                        time.sleep(1)

                    break
                else:
                    print("   Fail, Sleep for 1 seconds.")
                    time.sleep(1)

            except Exception as e:
                print("Exception was occurs.")
                print(e)
                continue
    print("End.")


if __name__ == "__main__":
    main()
