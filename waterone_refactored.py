import csv
import datetime
import os
import random
import sqlite3
import time

import pandas as pd
import requests


def crawl_stockid_table():
    """ 到證交所抓取股票代號 """

    twse_market_link = "https://isin.twse.com.tw/isin/C_public.jsp?strMode=2"
    otc_market_link = "https://isin.twse.com.tw/isin/C_public.jsp?strMode=4"

    myheader = {"user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                AppleWebKit/537.36 (KHTML, like Gecko) \
                Chrome/75.0.3770.142 Safari/537.36"}

    column_names = ["代號", "名稱", "上市日", "市場別", "產業別", "CFICode"]
    stock_table = pd.DataFrame(columns=column_names)

    try:
        for link in [twse_market_link, otc_market_link]:
            r = requests.get(link, headers=myheader)
            df = pd.read_html(r.text)[0]
            df.columns = [
                "有價證券代號及名稱", "國際證券辨識號碼(ISIN Code)", "上市日", "市場別", "產業別", "CFICode", "備註"]
            df = df[df["CFICode"] == "ESVUFR"].reset_index()
            df[["代號", "名稱"]] = df["有價證券代號及名稱"].str.split(expand=True)
            df.drop(["有價證券代號及名稱", "備註", "國際證券辨識號碼(ISIN Code)"], axis=1)
            df = df[["代號", "名稱", "上市日", "市場別", "產業別", "CFICode"]]
            stock_table = pd.concat([stock_table, df], axis=0)
    except:
        print('請檢查網路連線。')

    stock_table.to_csv(f'stock_table_{datetime.date.today()}.csv', index=False)
    return stock_table


def get_all_stockid():
    """ 取得所有股票的代號 """

    today = datetime.date.today()
    filename = f'stock_table_{today}.csv'
    if os.path.isfile(filename):
        print('讀取股票代號檔案')
        df = pd.read_csv(
            filename,
            encoding='utf-8',
            sep=',',
            thousands=',',
            dtype={'代號': str}
        )
        df = df[df['CFICode'] == 'ESVUFR']
        df = df.drop(['上市日', '市場別', '產業別', 'CFICode'], axis=1)
    else:
        print('股票代號檔案不存在！先從證交所抓，再讀入')
        crawl_stockid_table()
        df = pd.read_csv(
            filename,
            encoding='utf-8',
            sep=',',
            thousands=',',
            dtype={'代號': str}
        )
        df = df[df['CFICode'] == 'ESVUFR']
        df = df.drop(['上市日', '市場別', '產業別', 'CFICode'], axis=1)
    return df


def read_data_from_db(end_date=None, data_period=100):
    '''從資料庫讀取交易資料
    參數說明：
        最後一天的日期 end_date，預設為今天
        取用的資料天數 data_period，預設為 100 天
    '''
    if end_date is None:
        end_date = datetime.date.today()
    else:
        end_date = datetime.datetime.strptime(end_date,'%Y-%m-%d')
    # 計算 data_period 前的日期
    start_date = end_date - datetime.timedelta(days=data_period)
    start_date = start_date.strftime('%Y-%m-%d')
    end_date = end_date.strftime('%Y-%m-%d')

    # 連接資料庫
    conn = sqlite3.connect('stocks.sqlite')
    sql = f'SELECT * FROM OHLCV WHERE Date > "{start_date}" AND Date <= "{end_date}";'
    data = pd.read_sql(sql, conn)
    conn.close
    data = data.replace('--', 0)
    data = data.replace('---', 0)
    data = data.replace(' ---', 0)
    data = data.apply(lambda x: pd.to_numeric(
        x.astype(str).str.replace(',', ''), errors='ignore'))

    return data


def bollinger_calculate(df_stock, BOLLINGER_PERIOD=20):
    """Calculate the Bollinger upper bound, lower bound, bandwidth and volumes.
    """

    df_stock['UBand'] = df_stock['Close'].rolling(BOLLINGER_PERIOD).mean(
    ) + 2 * df_stock['Close'].rolling(BOLLINGER_PERIOD).std()
    df_stock['DBand'] = df_stock['Close'].rolling(BOLLINGER_PERIOD).mean(
    ) - 2 * df_stock['Close'].rolling(BOLLINGER_PERIOD).std()
    df_stock['BW'] = (df_stock['UBand'] -
                      df_stock['DBand']) / df_stock['DBand']
    df_stock['MovingVolume'] = df_stock['Volume'].rolling(5).mean()

    return df_stock


def is_over_upper_bound(df_stock) -> bool:
    last_row_number = len(df_stock) - 1
    condition_1 = df_stock.at[last_row_number,
                              'Close'] > df_stock.at[last_row_number, 'UBand']
    condition_2 = df_stock.at[last_row_number-1,
                              'Close'] < df_stock.at[last_row_number-1, 'UBand']
    condition_3 = df_stock.at[last_row_number,
                              'Close'] > df_stock.at[last_row_number, 'Open']
    if all([condition_1, condition_2, condition_3]):
        return True
    else:
        return False


def is_enough_volume(df_stock) -> bool:
    last_row_number = len(df_stock) - 1
    condition_1 = df_stock.at[last_row_number, 'Volume'] > 3 * \
        df_stock.at[last_row_number, 'MovingVolume']
    condition_2 = df_stock.at[last_row_number, 'Volume'] < 10 * \
        df_stock.at[last_row_number, 'MovingVolume']
    if all([condition_1, condition_2]):
        return True
    else:
        return False


def is_squeezed(df_stock, SQUEEZE_DAYS=60, BANDWIDTH_LIMIT=0.25) -> bool:
    last_row_number = len(df_stock) - 1
    condition = (df_stock.iloc[last_row_number -
                               SQUEEZE_DAYS:, 11].max() < BANDWIDTH_LIMIT)

    if condition:
        return True
    else:
        return False


def is_flow_quickly(df_stock, TRANSACTION_VALUE_LIMIT=10000000) -> bool:
    last_row_number = len(df_stock) - 1
    condition = (df_stock.at[last_row_number, 'Value']
                 > TRANSACTION_VALUE_LIMIT)

    if condition:
        return True
    else:
        return False


def is_buy(df_stock):
    '''
    參數說明：
        布林通道週期 BOLLINGER_PERIOD
        壓縮的天數 SQUEEZE_DAYS
        布林帶寬的限制 BANDWIDTH_LIMIT
        成交金額門檻 TRANSACTION_VALUE_LIMIT
        交易資料欄位順序：序號、日期、代碼、名稱、開、高、低、收、量、值、上、下、寬、均量
    '''

    # 檢查 df_stock 的資料是否夠多；最後一筆收盤價如為 0 則可能當日無法交易。
    try:
        if all([len(df_stock) > 60, df_stock.at[len(df_stock)-1, 'Close'] != 0]):
            df_stock = bollinger_calculate(df_stock)
            # 設定 4 層濾網
            # 起漲，收盤價超過上通道，且收紅 K
            filter1 = is_over_upper_bound(df_stock)

            # 帶量，起漲日的成交量超過前 5 日均量的三倍到十倍
            filter2 = is_enough_volume(df_stock)

            # 壓縮
            filter3 = is_squeezed(df_stock)

            # 流動性，成交金額超過 1 千萬
            filter4 = is_flow_quickly(df_stock)

            # if mask1 and mask2 and mask3 and mask4:
            if all([filter1, filter2, filter3, filter4]):
                return True
            else:
                return False
    except:
        return False


def read_entered(filename='阿水一式名單.csv'):
    entered = []
    with open(filename, encoding='utf-8-sig') as csvfile:
        rows = csv.DictReader(csvfile)
        for row in rows:
            entered.append(row)
    return entered


def sell_test(record, df_stock, loss_rate=-0.05):
    reasons = []

    # 出場指標
    # 向下跳空，開盤價低於前一個交易日的收盤價
    out1 = (df_stock.iloc[-1, 3] < df_stock.iloc[-2, 3]
            ) and (df_stock.iloc[-1, 3] < df_stock.iloc[-2, 6])

    # 虧損超過 loss_rate
    earn_loss = float(df_stock.iloc[-1, 6]) - float(record['買進價'])
    earn_rate = earn_loss / float(record['買進價'])
    out2 = earn_rate < loss_rate

    # MA5 開始下彎，利用均線扣抵
    out3 = df_stock.iloc[-1, 6] < df_stock.iloc[-6, 6]

    try:
        # 歸納原因
        if out1:
            reasons.append('開盤跳空下跌')
        if out2:
            reasons.append('停損')
        if out3:
            reasons.append('五日均線開始下彎')
    except:
        pass

    if reasons:
        return True, reasons
    else:
        return False, reasons
