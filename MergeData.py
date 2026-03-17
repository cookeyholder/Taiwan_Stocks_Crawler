import pandas as pd
import glob
import requests
import time
from io import StringIO
from datetime import datetime
import os


def FindEmptyDates():
    lists_twse = set([x[:10] for x in glob.glob('*上市公司股票成交資訊.csv')])
    lists_otc = set([x[:10] for x in glob.glob('*上櫃公司股票成交資訊.csv')])
    emptyDates = list(lists_otc^lists_twse)
    return emptyDates

def crawl_price(date):
    r = requests.get('http://www.twse.com.tw/exchangeReport/MI_INDEX?response=csv&date=' + str(date).replace('-','') + '&type=ALL')
    df_twse = pd.read_csv(StringIO("\n".join([i.translate({ord(c): None for c in ' '}) 
                                        for i in r.text.split('\n') 
                                        if len(i.split('",')) == 17 and i[0] != '='])), header=0)
    df_twse = df_twse.set_index('證券代號')
    df_twse['成交金額'] = df_twse['成交金額'].str.replace(',','')
    df_twse['成交股數'] = df_twse['成交股數'].str.replace(',','')
    df_twse.to_csv(date.strftime('%Y-%m-%d') + '上市公司股票成交資訊.csv', encoding='utf-8')
    print('已輸出到', date.strftime('%Y-%m-%d') + '上市公司股票成交資訊.csv')
    print('')
    return df_twse

def otc_crawler(date):
    otc_datalink = 'http://www.tpex.org.tw/web/stock/aftertrading/daily_close_quotes/stk_quote_download.php?l=zh-tw&d=' + str(date.year-1911) + date.strftime('/%m/%d') + '&s=0,asc,0'
    r = requests.get(otc_datalink)    
    lines = r.text.replace('\r', '').split('\n')    
    df_otc = pd.read_csv(StringIO("\n".join([line for line in lines if len(line.split('","')[0]) ==5] )), sep=',' , thousands=r',' , delimiter=',', header=None)
    df_otc = df_otc.sort_values(0).dropna()
    df_otc.columns = ['證券代號' , '證券名稱' , '收盤價' , '漲跌(+/-)' , '開盤價' , '最高價' , '最低價' , '平均價' , '成交股數' , '成交金額' , '成交筆數' , '最後揭示買價' , '最後揭示賣價' , '發行股數' , '次日參考價' , '次日漲停價' , '次日跌停價']
    df_otc = df_otc[['證券代號', '證券名稱', '成交股數', '成交筆數', '成交金額', '開盤價', '最高價', '最低價', '收盤價', '漲跌(+/-)', '最後揭示買價' , '最後揭示賣價', '平均價', '發行股數', '次日參考價', '次日漲停價', '次日跌停價']]
    df_otc = df_otc.set_index('證券代號')
    df_otc.to_csv(date.strftime('%Y-%m-%d') + '上櫃公司股票成交資訊.csv', encoding='utf-8')
    print('已輸出到', date.strftime('%Y-%m-%d') + '上櫃公司股票成交資訊.csv')
    print('')
    return df_otc

def MergeData():
    FileDates = [x[:10] for x in glob.glob('*上市公司股票成交資訊.csv')]
    
    for date in FileDates:
        print('正在合併' + date + '的資料。')
        # 讀入原始資料
        df_twse = pd.read_csv(date + '上市公司股票成交資訊.csv', encoding='utf-8', index_col='證券代號')
        df_otc = pd.read_csv(date + '上櫃公司股票成交資訊.csv', encoding='utf-8', index_col='證券代號')

        # 只取「開高低收量額」
        df_twse = df_twse[['證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數', '成交金額']]
        df_otc = df_otc[['證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數', '成交金額']]
	
        df = pd.concat([df_twse, df_otc], axis=0)
        df.to_csv(date + '臺股開高低收量額.csv')
        
    print('共', len(FileDates), '筆資料。')
            
if len(FindEmptyDates())==0:
    MergeData()
else:
    print('上市和上櫃的資料數量不一致！')
    print('以下日期的資料不一致：')
    lists_twse = set([x[:10] for x in glob.glob('*上市公司股票成交資訊.csv')])
    lists_otc = set([x[:10] for x in glob.glob('*上櫃公司股票成交資訊.csv')])
    print(lists_twse^lists_otc)
    
    for date in lists_twse - lists_otc:
        print('正在抓缺漏的' + date +'上櫃公司股票成交資訊')
        newdf = otc_crawler(datetime.strptime(date, '%Y-%m-%d'))
        time.sleep(1)  # 休息 1 秒鐘
    
    for date in lists_otc - lists_twse:
        print('正在抓缺漏的' + date + '上市公司股票成交資訊')
        newdf = twse_crawler(date)
        time.sleep(1)  # 休息 1 秒鐘
    MergeData()
