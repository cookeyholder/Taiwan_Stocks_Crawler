# -*- coding: utf-8 -*-
"""
Created on Thu Dec 26 14:52:14 2019

@author: slwang
"""


import pandas as pd
import requests
import datetime


def GetAllStockID():
    df = pd.read_csv('StockID.csv', encoding='utf-8', sep=',', thousands=',', dtype={'代號':str})
    df = df[df['CFICode'] == 'ESVUFR']
    df = df[df['市場別'] == '上櫃']
    df = df.drop(['上市日', '市場別', '產業別', 'CFICode'], axis=1)
    return df


def Date2Taiwandate(date):
    year = str(int(date.strftime('%Y')) -1911)
    month = date.strftime('%m')
    day = date.strftime('%d')
    return year + '/' + month + '/' + day


def otc_crawler(date):
    url = 'https://www.tpex.org.tw/web/stock/aftertrading/otc_quotes_no1430/stk_wn1430_result.php?l=zh-tw&o=htm&d=' + Date2Taiwandate(date) + '&se=EW&s=0,asc,0'
    r = requests.get(url)
    df= pd.read_html(r.text)[0]
    df.columns = ['證券代號', '證券名稱', '收盤價', '漲跌', '開盤價', '最高價', '最低價', '成交股數', '成交金額(元)', '成交筆數'	, '最後買價', '最後賣價', '發行股數', '次日漲停價', '次日跌停價']
    df = df[['證券代號', '證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數']]
    df = df[df['證券代號'].isin(GetAllStockID()['代號'])]
    return df


ids = GetAllStockID()
df = otc_crawler(datetime.datetime.today())
df.to_csv('abc.csv', index=False)