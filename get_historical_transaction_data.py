'''
取得上市及上櫃的每日收盤行情，並透過 LINE Notify 功能通知是否完成。
'''

import datetime
import glob
import json
import os
import random
import sqlite3
import time

import pandas as pd
import requests
from fake_useragent import UserAgent

from you import Line, Stocks, youProxies

# 阿財 token
money_token = 'BUQDH5qA4hmxNwev6zQUjR1bsLOg1hOFIVbfpgbZmjU'

# Vanessa token
Vanessa_token = 'LMdb0htDvZOLd9MUCZz31iwCTCme7H7VMgxeW0UFxSY'

def get_historical_transaction_data(end_date=datetime.date.today()):
    if (Line.lineNotifyMessage(Vanessa_token, '開始抓取歷史收盤資料')) != 200:
        print('LINE 訊息傳送有問題。')

    # 計算已有資料的最後一天
    fn_list = glob.glob(os.path.join('每日收盤行情', '*開高低收量額.csv'))
    if len(fn_list) != 0:
        start_date = fn_list[-1].split('臺')[0][-10:]
    else:
        start_date = '2008-01-01'
    
    # 將已有資料的最後一天的日期加 1 天
    start_date = datetime.datetime.strptime(start_date, '%Y-%m-%d') + datetime.timedelta(days=1)

    # 將起迄日期轉成 ISO String
    start_date = start_date.strftime('%Y-%m-%d')
    end_date = end_date.strftime('%Y-%m-%d')

    # 產生所有從已有資料的最後一天到今天的所有日期
    date_list = pd.date_range(start_date, end_date)
    day_counter = 0

    for trade_date in date_list:
        print(f'正在抓取 {trade_date.strftime("%Y-%m-%d")} 的資料...')
        df_otc = Stocks.otc_crawler(trade_date)
        if df_otc is not False:
            df_twse = Stocks.twse_crawler(trade_date)
            trade_date = trade_date.strftime('%Y-%m-%d')
            df = pd.concat([df_twse, df_otc])
            df['Symbol'] = df['Symbol'].astype('str')
            df = df.sort_values(by=['Symbol']).reset_index(drop=True)
            df.to_csv(os.path.join('每日收盤行情', f'{trade_date}臺股開高低收量額.csv'), encoding='utf8', index=False)
            print(f'        抓取 {trade_date} 的資料結束')
        else:
            print(f'        {trade_date.strftime("%Y-%m-%d")} 沒有交易')
            print(f'        抓取 {trade_date.strftime("%Y-%m-%d")} 的資料結束')
            continue
        
        # 寫入到資料庫
        Stocks.write_to_db(df)
        # 休息 5 到 10 秒鐘再抓
        time.sleep(random.randint(5, 10))
        day_counter = day_counter + 1
    
    msg = f'下載每日收盤行情完成，總共抓取了 {day_counter} 天的交易資料。'
    if (Line.lineNotifyMessage(Vanessa_token, msg)) != 200:
        msg = '每日收盤行情下載失敗'
        print(msg)
    
    print('全部結束。')
    print(f'總共抓取了 {day_counter} 天的交易資料。')


if __name__ == '__main__':
    get_historical_transaction_data()
