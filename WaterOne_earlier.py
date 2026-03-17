'''
阿水提早名單篩選
預計執行時間：14:00
盤後交易時間：14:00 到 14:30
'''

import csv
import datetime
import os
import sqlite3
import time

import numpy as np
import pandas as pd
import requests

from you import Line, Stocks, WaterOne

def main():
    # 定義欄位
    fieldnames = [
        '交易序號',
        '買進日期',
        '股票代號',
        '股票名稱',
        '買進價',
        'Yahoo連結',
        '賣出日期',
        '賣出價',
        '盈虧',
        '報酬率',
        '出場原因'
        ]

    enter_result = [fieldnames]
    error_list = [['日期', '股票代碼']]

    StockID = WaterOne.GetAllStockID()
    stock_ids = list(StockID['代號'])
    stock_ids.sort()

    today = datetime.date.today()
    yesterday = today - datetime.timedelta(days=1)
    # 從資料庫讀取之前的交易資料，再與今天的交易資料合併, 進行進場檢驗
    all_stock_prices = WaterOne.read_data_from_db(end_date=yesterday)
    df_otc = Stocks.otc_crawler(today)
    
    if df_otc is not False:
        df_twse = Stocks.twse_crawler(today)
        today = today.strftime('%Y-%m-%d')
        df_today = pd.concat([df_twse, df_otc])
        df_today['Symbol'] = df_today['Symbol'].astype('str')
        df_today['Open'] = df_today['Open'].astype('float64')
        df_today['High'] = df_today['High'].astype('float64')
        df_today['Low'] = df_today['Low'].astype('float64')
        df_today['Close'] = df_today['Close'].astype('float64')
        df_today['Volume'] = df_today['Volume'].astype('int64')
        df_today['Value'] = df_today['Value'].astype('int64')
        all_stock_prices = pd.concat([all_stock_prices, df_today])

    all_stock_prices.to_csv(f'{today}前之交易資料(檢查waterone_earlier用).csv', encoding='utf-8-sig')

    # 開始篩選名單
    print(f'正在篩選 {today} 符合條件的個股')
    for stock_id in stock_ids:
        try:
            df_stock = all_stock_prices[all_stock_prices['Symbol'] == stock_id].reset_index(drop=True)
            if len(df_stock) > 60:
                tmp = WaterOne.buy_test(df_stock)
                if len(tmp) > 1:
                    enter_result.append(tmp)
        except:
            error_list.append([today, stock_id])
            continue

    if enter_result:
        tmp_result = pd.DataFrame(enter_result, columns=enter_result.pop(0))
        tmp_result.to_csv('阿水一式名單.csv', mode='a+', index=False, header=False)

    # LINE 訊息通知
    # enter_result 的標題列已經 pop 走了
    enter_msg = f'\n阿水一式進場條件：\n60天內布林帶寬小於 5%\n成交量超過五日均量的三倍\n紅K突破上通道\n\n{today} 滿足阿水一式的進場名單：\n'
    if enter_result:
        for row in enter_result:
            if row[1] == today:
                enter_msg = enter_msg + f'{row[3]}({row[2]})\n{row[5]}\n\n'
    else:
        enter_msg = enter_msg + '沒有進場名單'

    # 休息 2 秒鐘
    time.sleep(2)


    # 阿水一式的出場檢查
    for_checks = WaterOne.read_entered()
    sell_msg = f'\n阿水一式的出場條件：\n開盤跳空下跌\n虧損超過 5%\n五日均線開始下彎\n\n{today} 滿足阿水一式的出場名單：\n'
    for i, check in enumerate(for_checks):
        if check['賣出日期'] == '' and check['買進日期'] != today:
            stock_id = check['股票代號']
            # stock_filter = (all_stock_prices['Symbol'] == stock_id)
            df_stock = all_stock_prices[all_stock_prices['Symbol'] == stock_id].reset_index(drop=True)
            isSell, reasons = WaterOne.sell_test(check, df_stock)
            if isSell == True:
                for_checks[i]['賣出日期'] = df_stock.iloc[-1, 0]
                for_checks[i]['賣出價'] = float(df_stock.iloc[-1, 6])
                for_checks[i]['盈虧'] = float(for_checks[i]['賣出價']) - float(for_checks[i]['買進價'])
                for_checks[i]['報酬率'] = float(for_checks[i]['盈虧']) / float(for_checks[i]['買進價'])
                for_checks[i]['出場原因'] = reasons
                sell_msg = sell_msg + f"{check['股票名稱']}({check['股票代號']})\n{check['出場原因']}\n買進價{check['買進價']}\n賣出價{check['賣出價']}\n報酬率{format(check['報酬率']*100, '.2g')}%\n{check['Yahoo連結']}\n\n"

    with open('阿水一式名單.csv', 'w', encoding='utf-8-sig', newline='') as csvfile:
        # 將 dictionary 寫入 CSV 檔
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)

        # 寫入第一列的欄位名稱
        writer.writeheader()

        for check in for_checks:
            # 寫入資料
            writer.writerow(check)

    # 休息 2 秒鐘
    time.sleep(2)



    # 阿水一式未出場報酬率
    for_checks = WaterOne.read_entered()
    rate_msg = f'\n{today} 未出場的報酬率：\n'
    for check in for_checks:
        # 如果「賣出日期」是空白，表示未賣出；略過今天才買進的個股。
        if (check['賣出日期'] == '') and (check['買進日期'] != today):
            stock_id = check['股票代號']
            stock_filter = (all_stock_prices['Symbol'] == stock_id)
            df_stock = all_stock_prices[stock_filter].reset_index(drop=True)
            isSell, _ = WaterOne.sell_test(check, df_stock)
            if isSell == False:
                return_rate = (float(df_stock.iloc[-1, 6]) - float(check['買進價']))/float(check['買進價'])
                rate_msg = rate_msg + f"{check['股票名稱']}({check['股票代號']})\n買進價格{check['買進價']}\n現在價格{float(df_stock.iloc[-1, 6])}\n報酬率{format(return_rate, '.2%')}\n{check['Yahoo連結']}\n\n"

    # 發送進場簡訊
    time.sleep(2)
    if Line.Money(enter_msg) != 200:
        print('Line Notify has some problem.')
        
    # 發送出場簡訊
    time.sleep(2)
    if Line.Money(sell_msg) != 200:
        print('Line Notify has some problem.')

    # 發送未出場報酬率
    time.sleep(2)
    if Line.Money(rate_msg) != 200:
        print('Line Notify has some problem.')
    
    
if __name__ == '__main__':
    main()