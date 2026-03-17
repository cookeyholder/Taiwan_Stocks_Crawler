
# coding: utf-8

import requests
import pandas as pd
import glob
import time
import random
from io import StringIO
from datetime import datetime


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


def Output2File():
    # 用一個空 dict 裝爬到的資料，以日期當 key，一張 DataFrame 當 value
    dailyprices = {file[:10]:pd.read_csv(file, encoding='utf-8', index_col='證券代號') for file in glob.glob('*上櫃公司股票成交資訊.csv')  
    
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

    #   寫入 CSV 檔
    close.to_csv('上櫃公司股票收盤價格.csv')
    openprice.to_csv('上櫃公司股票開盤價格.csv')
    high.to_csv('上櫃公司股票最高價格.csv')
    low.to_csv('上櫃公司股票最低價格.csv')
    volume.to_csv('上櫃公司股票成交股數.csv')


# 取得已有的最新資料日期
# 如果沒有之前的資料，則設定最早的日期為 2007-04-22
# 2001-04-23 是櫃買中心有提供資料的最早日期
while True:
    try:
        NewestDate = glob.glob('*上櫃公司股票成交資訊.csv')[-1][:10]
        NewestDate = datetime.strptime(NewestDate.replace('-', ''), '%Y%m%d').date()
        break
    except IndexError:
        NewestDate = '2007-04-22'
        NewestDate = datetime.strptime(NewestDate.replace('-', ''), '%Y%m%d').date()
        break

# 從已有的最新資料日開始抓取資料，直到與現在日期相同時停止
CrawlDate = NewestDate + datetime.timedelta(days=1)

now = datetime.now()
Time1330 = now.replace(hour=13, minute=30, second=0, microsecond=0)
if now < Time1330:
    TodayDate = datetime.today()
else:
    TodayDate = datetime.today() + datetime.timedelta(days=1)

success_counter = 1
fail_counter = 0
allow_continuous_fail = 20  # 最大失敗次數

while CrawlDate.strftime('%Y-%m-%d') != TodayDate.strftime('%Y-%m-%d'):
    # 使用 otc_crawler 爬資料
    try:
        # 抓資料
        df = otc_crawler(CrawlDate)
        print('現在時間為', datetime.now().strftime('%Y-%m-%d %H:%M:%S'))
        print('正在抓取', CrawlDate.strftime('%Y-%m-%d'), '的上櫃市場交易資料')
        print('這是第', success_counter, '筆資料。')
        print(CrawlDate.strftime('%Y-%m-%d'), '成功! 已連續失敗', fail_counter, '次')
        success_counter += 1
        fail_counter = 0
    except:
        # 假日爬不到
        print('現在時間為', datetime.now().strftime('%Y-%m-%d %H:%M:%S'))
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
        time.sleep(random.randint(8, 10))  # 每成功 10 次，隨機休息 8 到 10 秒
        print('休息結束')
        print('')
    else:
        print('小休息中……')
        time.sleep(random.randint(5, 8))  # 隨機休息 5 到 8 秒
        print('休息結束')
        print('')

#Output2File()