import csv
import datetime
import os
import time

import pandas as pd
import requests
from fake_useragent import UserAgent


def get_query_dates():
    '''
    取得股權分散表選單中的所有日期
    '''
    do_url = 'https://www.tdcc.com.tw/smWeb/QryStockAjax.do'
    datas = {'REQ_OPR': "qrySelScaDates"}
    my_header = {'user-agent': UserAgent().random}
    r = requests.post(do_url, headers=my_header, data=datas)
    dates = r.text.replace('["', '').replace('"]', '').split('","')
    return dates


def get_stock_ownership(stock_id):
    '''
    取得臺灣集中保管結算所的個股股權分散表
    '''
    end_date = datetime.date.today()
    start_date = end_date - datetime.timedelta(weeks=52)
    query_dates = get_query_dates()
    for i, date in enumerate(query_dates):
        print('{:>02d}.'.format(i), 'Crawling date is', date)
        while(True):
            try:
                url = 'https://www.tdcc.com.tw/smWeb/QryStockAjax.do'
                payload={
                    'scaDates':date,
                    'scaDate':date,
                    'SqlMethod':'StockNo',
                    'StockNo':stock_id,
                    'radioStockNo':stock_id,
                    'StockName':'',
                    'REQ_OPR':'SELECT',
                    'clkStockNo':stock_id,
                    'clkStockName':''
                    }

                my_header = {'user-agent': UserAgent().random}
                r = requests.post(url, data=payload, headers=my_header)
                r.encoding = 'big5-hkscs'
                stock_name = pd.read_html(r.text)[5].iloc[0, 0].split('證券名稱：')[1]
                df = pd.read_html(r.text)[6]
                columns = df.iloc[0,:].to_list()
                df.columns = columns
                df = df.drop([0])
                if df.iloc[0, 0] != '無此資料':
                    filename = f'{stock_id}({stock_name}){date}股權分散表.csv'
                    df.to_csv(filename, encoding='utf-8', index=False)
                    print('   Success, Sleep for 1 seconds.')
                    time.sleep(1)
                    break
                else:
                    print('   Fail, Sleep for 1 seconds.')
                    time.sleep(1)

            except Exception as e:
                print('Exception was occurs.')
                print(e)
                continue
    print('End.')
