import csv
import datetime
import os
import random
import sqlite3
import time

import pandas as pd
import requests


# 到證交所抓取股票代號
def crawlStockID():
    twse_market = "https://isin.twse.com.tw/isin/C_public.jsp?strMode=2"
    otc_market = "https://isin.twse.com.tw/isin/C_public.jsp?strMode=4"

    myheader = {
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                AppleWebKit/537.36 (KHTML, like Gecko) \
                Chrome/75.0.3770.142 Safari/537.36"
    }
    stockcodes = pd.DataFrame(columns=["代號", "名稱", "上市日", "市場別", "產業別", "CFICode"])
    try:
        for market in [twse_market, otc_market]:
            r = requests.get(market, headers=myheader)
            df = pd.read_html(r.text)[0]
            df.columns = [
                "有價證券代號及名稱",
                "國際證券辨識號碼(ISIN Code)",
                "上市日",
                "市場別",
                "產業別",
                "CFICode",
                "備註",
            ]
            df = df[df["CFICode"] == "ESVUFR"].reset_index()
            df[["代號", "名稱"]] = df["有價證券代號及名稱"].str.split(expand=True)
            df.drop(["有價證券代號及名稱", "備註", "國際證券辨識號碼(ISIN Code)"], axis=1)
            df = df[["代號", "名稱", "上市日", "市場別", "產業別", "CFICode"]]
            stockcodes = pd.concat([stockcodes, df], axis=0)
    except:
        print("請檢查網路連線。")

    stockcodes.to_csv("StockID.csv", index=False)
    return stockcodes


# 取得所有上市櫃股票的代號


def GetAllStockID():
    filectime = time.localtime(os.stat("StockID.csv").st_mtime)
    today = datetime.date.today()
    filenotexist = not os.path.isfile("StockID.csv")
    filenotupdate = (filectime.tm_mon, filectime.tm_mday) != (today.month, today.day)

    if filenotexist:
        print("股票代號檔案不存在！先從證交所抓，再讀入")
        crawlStockID()
        df = pd.read_csv(
            "StockID.csv", encoding="utf-8", sep=",", thousands=",", dtype={"代號": str}
        )
        df = df[df["CFICode"] == "ESVUFR"]
        df = df.drop(["上市日", "市場別", "產業別", "CFICode"], axis=1)
        # df = df.set_index('代號')
    elif filenotupdate:
        print("股票代號檔案太舊，從證交所更新股票代號")
        crawlStockID()
        df = pd.read_csv(
            "StockID.csv", encoding="utf-8", sep=",", thousands=",", dtype={"代號": str}
        )
        df = df[df["CFICode"] == "ESVUFR"]
        df = df.drop(["上市日", "市場別", "產業別", "CFICode"], axis=1)
        # df = df.set_index('代號')
    else:
        print("讀取股票代號檔案")
        df = pd.read_csv(
            "StockID.csv", encoding="utf-8", sep=",", thousands=",", dtype={"代號": str}
        )
        df = df[df["CFICode"] == "ESVUFR"]
        df = df.drop(["上市日", "市場別", "產業別", "CFICode"], axis=1)
        # df = df.set_index('代號')

    return df


# 從資料庫讀取交易資料


def read_data_from_db(end_date=datetime.date.today(), data_period=100):
    """
    參數說明：
        最後一天的日期 end_date，預設為今天
        取用的資料天數 data_period，預設為 100 天
    """

    # 計算 data_period 前的日期
    start_date = end_date - datetime.timedelta(days=data_period)
    start_date = start_date.strftime("%Y-%m-%d")
    end_date = end_date.strftime("%Y-%m-%d")

    # 連接資料庫
    conn = sqlite3.connect("stocks.sqlite")
    sql = f'SELECT * FROM OHLCV WHERE Date > "{start_date}" AND Date <= "{end_date}";'
    df = pd.read_sql(sql, conn)
    conn.close
    df = df.replace("--", 0)
    df = df.replace("---", 0)
    df = df.replace(" ---", 0)
    df = df.apply(lambda x: pd.to_numeric(x.astype(str).str.replace(",", ""), errors="ignore"))

    return df


def buy_test(
    df_stock,
    bollinger_period=20,
    squeeze_days=60,
    bandwidth_limit=0.4,
    transaction_value_limit=10000000,
):
    """
    參數說明：
        布林通道週期 bollinger_period
        壓縮的天數 squeeze_days
        布林帶寬的限制 bandwitdh_limit
        成交金額門檻 transaction_value_limit
        交易資料欄位順序：序號、日期、代碼、名稱、開、高、低、收、量、值、上、下、寬、均量
    """
    yahoo_prefix = "https://tw.stock.yahoo.com/q/ta?s="
    goodinfo_prefix = "https://goodinfo.tw/StockInfo/ShowK_Chart.asp?STOCK_ID="

    # 交易序號長度為 20 碼，年月日時分秒(14碼) + 6位數的隨機整數
    serial = datetime.datetime.now().strftime("%Y%m%d%H%M%S") + str(random.randint(100000, 999999))
    result = [serial]

    # 檢查 df_stock 的資料是否夠多
    if len(df_stock) > 60:
        df_stock["UBand"] = (
            df_stock["Close"].rolling(bollinger_period).mean()
            + 2 * df_stock["Close"].rolling(bollinger_period).std()
        )
        df_stock["DBand"] = (
            df_stock["Close"].rolling(bollinger_period).mean()
            - 2 * df_stock["Close"].rolling(bollinger_period).std()
        )
        df_stock["BW"] = (df_stock["UBand"] - df_stock["DBand"]) / df_stock["DBand"]
        df_stock["MovingVolume"] = df_stock["Volume"].rolling(5).mean()

        # 設定 4 層濾網
        # 起漲，收盤價超過上通道，且收紅 K
        mask1 = (
            (df_stock.iloc[-1, 6] > df_stock.iloc[-1, 9])
            and (df_stock.iloc[-2, 6] < df_stock.iloc[-2, 9])
            and (df_stock.iloc[-1, 6] > df_stock.iloc[-1, 3])
        )

        # 帶量，起漲日的成交量超過前 5 日均量的三倍到十倍
        mask2 = (df_stock.iloc[-1, 7] > 3 * df_stock.iloc[-1, 12]) and (
            df_stock.iloc[-1, 7] < 10 * df_stock.iloc[-1, 12]
        )

        # 壓縮，布林帶寬 < 5%
        mask3 = df_stock.iloc[-squeeze_days:, 11].max() < bandwidth_limit

        # 流動性，成交金額超過 1 千萬
        mask4 = df_stock.iloc[-1, 8] > transaction_value_limit

        # if mask1 and mask2 and mask3 and mask4:
        if all([mask1, mask2, mask3, mask4]):
            if df_stock.iloc[-1, 6] != 0:
                result = (
                    result + df_stock[["Date", "Symbol", "Name", "Close"]].iloc[-1, :].to_list()
                )
                result = result + [goodinfo_prefix + result[2], "", "", "", "", ""]

    return result


def read_entered(fn="阿水一式名單.csv"):
    entered = []
    with open(fn, encoding="utf-8-sig") as csvfile:
        rows = csv.DictReader(csvfile)
        for row in rows:
            entered.append(row)
    return entered


def sell_test(record, df_stock, loss_rate=-0.05):
    reasons = []

    # 出場指標
    # 向下跳空，開盤價低於前一個交易日的收盤價
    out1 = (df_stock.iloc[-1, 3] < df_stock.iloc[-2, 3]) and (
        df_stock.iloc[-1, 3] < df_stock.iloc[-2, 6]
    )

    # 虧損超過 loss_rate
    earn_loss = float(df_stock.iloc[-1, 6]) - float(record["買進價"])
    earn_rate = earn_loss / float(record["買進價"])
    out2 = earn_rate < loss_rate

    # MA5 開始下彎，利用均線扣抵
    out3 = df_stock.iloc[-1, 6] < df_stock.iloc[-6, 6]

    try:
        # 歸納原因
        if out1:
            reasons.append("開盤跳空下跌")
        if out2:
            reasons.append("停損")
        if out3:
            reasons.append("五日均線開始下彎")
    except:
        pass

    if reasons:
        return True, reasons
    else:
        return False, reasons


"""     if len(reasons) == 0:
        return False, reasons
    else:
        return True, reasons """
