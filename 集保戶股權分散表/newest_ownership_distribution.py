import csv
import time

import pandas as pd
import requests
from fake_useragent import UserAgent


def main():
    '''
    第1級至第15級，係持股為1-999 、1,000-5,000、5,001-10,000、10,001-15,000、15,001-20,000、20,001-30,000、30,001-40,000、40,001-50,000、50,001-100,000、100,001-200,000、200,001-400,000、400,001-600,000、600,001-800,000、800,001-1,000,000、1,000,001以上等15個級距。
    
    另考量交易市場偶因資料日前1營業日客戶帳戶賣出餘額不足之情事發生，使各「持股分級」合計股數與發行公司已發行股份總數產生之差異，爰新增第16欄差異數調整。
    第17欄為合計欄。
    '''
    while(True):
        url = 'https://smart.tdcc.com.tw/opendata/getOD.ashx?id=1-5'
        my_header = {'user-agent': UserAgent().random}
        try:
            r = requests.get(url, headers=my_header)
            r.encoding = 'utf-8-sig'
            ownership = []
            lines = r.text.split('\n')
            for line in lines:
                tmp = [x.strip() for x in line.split(',')]
                ownership.append(tmp)
            
            # 計算日期及檔名
            date = ownership[1][0]
            date = date[:4] + '-' + date[4:6] + '-' + date[-2:]
            filename = date + '集保戶股權分散表.csv'

            # 寫入檔案
            with open(filename, 'w', newline='', encoding='utf-8-sig') as csvfile:
                writer = csv.writer(csvfile)
                writer.writerows(ownership)

            if ownership:
                break
        except:
            time.sleep(10)
            continue


if __name__ == '__main__':
    main()
