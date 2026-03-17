# -*- coding: utf-8 -*-
"""
Created on Wed Dec 18 17:18:00 2019

@author: slwang
"""

# -*- coding: utf-8 -*-
"""
從現有最新的日期開始抓成交資訊，直到今天
"""


from io import StringIO
import datetime
import time
import random
import requests
import pandas as pd
import glob
import sqlite3


def twse_crawler(date):
    r = requests.get('http://www.twse.com.tw/exchangeReport/MI_INDEX?response=csv&date=' + str(date).replace('-','') + '&type=ALL')
    df_twse = pd.read_csv(StringIO("\n".join([i.translate({ord(c): None for c in ' '})
                                        for i in r.text.split('\n')
                                        if len(i.split('",')) == 17 and i[0] != '='])), header=0, encoding='big5-hkscs')
    df_twse = df_twse.set_index('證券代號')
    df_twse['成交金額'] = df_twse['成交金額'].str.replace(',','')
    df_twse['成交股數'] = df_twse['成交股數'].str.replace(',','')
    df_twse.to_csv(date.strftime('%Y-%m-%d') + '上市公司股票成交資訊.csv', encoding='utf-8-sig')
    print('已輸出到', date.strftime('%Y-%m-%d') + '上市公司股票成交資訊.csv')
    print('')
    return df_twse

def otc_crawler(date):
    otc_datalink = 'http://www.tpex.org.tw/web/stock/aftertrading/daily_close_quotes/stk_quote_download.php?l=zh-tw&d=' + str(date.year-1911) + date.strftime('/%m/%d') + '&s=0,asc,0'

    r = requests.get(otc_datalink)
    lines = r.text.replace('\r', '').split('\n')

    df_otc = pd.read_csv(StringIO("\n".join([line for line in lines if len(line.split('","')[0]) ==5] )), sep=',' , thousands=r',' , delimiter=',', header=None, encoding='big5-hkscs')
    df_otc = df_otc.sort_values(0).dropna()
    df_otc.columns = ['證券代號',
                      '證券名稱',
                      '收盤價',
                      '漲跌(+/-)',
                      '開盤價',
                      '最高價',
                      '最低價',
                      '平均價',
                      '成交股數',
                      '成交金額',
                      '成交筆數',
                      '最後揭示買價',
                      '最後買量(千股)',
                      '最後揭示賣價',
                      '最後賣量(千股)',
                      '發行股數',
                      '次日參考價',
                      '次日漲停價',
                      '次日跌停價']

    df_otc = df_otc[['證券代號',
                     '證券名稱',
                     '成交股數',
                     '成交筆數',
                     '成交金額',
                     '開盤價',
                     '最高價',
                     '最低價',
                     '收盤價',
                     '漲跌(+/-)',
                     '最後揭示買價',
                     '最後揭示賣價',
                     '平均價',
                     '發行股數',
                     '次日參考價',
                     '次日漲停價',
                     '次日跌停價']]

    df_otc = df_otc.set_index('證券代號')

    df_otc.to_csv(date.strftime('%Y-%m-%d') + '上櫃公司股票成交資訊.csv', encoding='utf-8-sig')
    print('已輸出到', date.strftime('%Y-%m-%d') + '上櫃公司股票成交資訊.csv')
    print('')

    return df_otc

def Output2File():
    # 用一個空 dict 裝爬到的資料，以日期當 key，一張 DataFrame 當 value
    dailyprices = {file[:10]:pd.read_csv(file, encoding='utf-8', index_col='證券代號') for file in glob.glob('*臺股開高低收量額.csv')}

    close = pd.DataFrame({k:df['收盤價'] for k,df in dailyprices.items()}).transpose()
    close.index = pd.to_datetime(close.index)

    openprice = pd.DataFrame({k:df['開盤價'] for k,df in dailyprices.items()}).transpose()
    openprice.index = pd.to_datetime(openprice.index)

    high = pd.DataFrame({k:df['最高價'] for k,df in dailyprices.items()}).transpose()
    high.index = pd.to_datetime(high.index)

    low = pd.DataFrame({k:df['最低價'] for k,df in dailyprices.items()}).transpose()
    low.index = pd.to_datetime(low.index)

    volume = pd.DataFrame({k:df['成交股數'] for k,df in dailyprices.items()}).transpose()
    volume.index = pd.to_datetime(volume.index)

    value = pd.DataFrame({k:df['成交金額'] for k,df in dailyprices.items()}).transpose()
    value.index = pd.to_datetime(volume.index)

    #   寫入 CSV 檔
    close.to_csv('臺股收盤價格.csv')
    openprice.to_csv('臺股開盤價格.csv')
    high.to_csv('臺股最高價格.csv')
    low.to_csv('臺股最低價格.csv')
    volume.to_csv('臺股成交股數.csv')
    value.to_csv('臺股成交金額.csv')


# 取得已有的最新資料日期
# 如果沒有之前的資料，則設定最早的日期為 2007-04-22
# 2001-04-23 是櫃買中心有提供資料的最早日期
while True:
    try:
        files = glob.glob('*臺股開高低收量額.csv')
        files.sort()
        NewestDate = files[-1][-22:-12]
        NewestDate = datetime.datetime.strptime(NewestDate.replace('-', ''), '%Y%m%d').date()
        break
    except IndexError:
        NewestDate = '2007-04-22'
        NewestDate = datetime.datetime.strptime(NewestDate.replace('-', ''), '%Y%m%d').date()
        break

# 從已有的最新資料日開始抓取資料，直到與現在日期相同時停止
CrawlDate = NewestDate + datetime.timedelta(days=1)

now = datetime.datetime.now()
Time1330 = now.replace(hour=13, minute=30, second=0, microsecond=0)
if now < Time1330:
    TodayDate = datetime.datetime.today()
else:
    TodayDate = datetime.datetime.today() + datetime.timedelta(days=1)

success_counter = 1
fail_counter = 0
allow_continuous_fail = 20  # 最大失敗次數


while CrawlDate.strftime('%Y-%m-%d') != TodayDate.strftime('%Y-%m-%d'):
    # 使用 twse_crawler 和 otc_crawler 爬資料
    try:
        # 抓資料
        print('現在時間為', datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S'))
        print('這是第', success_counter, '筆資料。')

        # 抓上市公司交易資料
        print('正在抓取', CrawlDate.strftime('%Y-%m-%d'), '的上市公司交易資料')
        df_twse = twse_crawler(CrawlDate)
        df_twse = df_twse[['證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數', '成交金額']]

        # 抓上櫃公司交易資料
        print('正在抓取', CrawlDate.strftime('%Y-%m-%d'), '的上櫃公司交易資料')
        df_otc = otc_crawler(CrawlDate)
        df_otc = df_otc[['證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數', '成交金額']]

        # 合併上市上櫃交易資料
        df_all = pd.concat([df_twse, df_otc], axis=0)
        df_all.to_csv(CrawlDate.strftime('%Y-%m-%d') + '臺股開高低收量額.csv', encoding='utf-8')
        df_all['日期'] = CrawlDate
        df_all['代號'] = df_all.index
        df_all = df_all[['日期', '代號', '證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數', '成交金額']]

        # 準備寫入到資料庫
        df_all.columns = ['Date', 'Symbol', 'Name', 'Open', 'High', 'Low', 'Close', 'Volume', 'Value']
        print('寫入到 stocks.sqlite 資料庫')
        conn = sqlite3.connect('stocks.sqlite')
        df_all.to_sql('OHLCV', conn, if_exists='append', index=False)
        conn.commit()
        conn.close()
        print('寫入資料庫完成')
        print(CrawlDate.strftime('%Y-%m-%d'), '成功! 已連續失敗', fail_counter, '次')

        success_counter += 1
        fail_counter = 0
    except:
        # 假日爬不到
        # print('現在時間為', datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S'))
        print(CrawlDate.strftime('%Y-%m-%d') + ' 是假日！', '之前已連續失敗', fail_counter, '次')
        print('')
        fail_counter += 1
        if fail_counter == allow_continuous_fail:
            raise
            break

    # 加一天
    CrawlDate += datetime.timedelta(days=1)
    if success_counter % 10 == 3:
        print('大休息中……')
        time.sleep(random.randint(15, 20))  # 每成功 10 次，隨機休息 15 到 20 秒
        print('休息結束')
        print('')
    else:
        print('小休息中……')
        time.sleep(random.randint(5, 10))  # 隨機休息 5 到 10 秒
        print('休息結束')
        print('')

Output2File()
