import csv
import datetime
import os

import pandas as pd
import requests
from fake_useragent import UserAgent

from you import Line

def ownership_distribution():
    '''
    取得集中保管所的股權分散表，所有證券的。
    '''
    url = 'https://smart.tdcc.com.tw/opendata/getOD.ashx?id=1-5'
    my_header = {'user-agent': UserAgent().random}
    while(True):
        try:
            r = requests.get(url, headers=my_header)
            r.encoding = 'utf-8-sig'

            ownership = []
            lines = r.text.split('\n')
            for line in lines:
                tmp = [x.strip() for x in line.split(',')]
                ownership.append(tmp)

            date = ownership[1][0]
            date = date[:4] + '-' + date[4:6] + '-' + date[-2:]
            filename = f'集保戶股權分散表{os.sep}' + date + '集保戶股權分散表.csv'
            with open(filename, 'w', newline='', encoding='utf-8-sig') as csvfile:
                writer = csv.writer(csvfile)
                writer.writerows(ownership)
            
                        # 發送 Line 訊息通知己下載完成
            msg = f'{date} 所有股票的股權分散表已經下載完成。'
            print(msg)
            Line.Vanessa(msg)
            break

        except Exception as e:
            print('Exception was occured.')
            print(e)
            continue

if __name__ == '__main__':
    ownership_distribution()