# -*- coding: utf-8 -*-
"""
Created on Sat Dec 28 16:34:09 2019

用來練習操作 SQLite 資料庫系統，
包括連接、關閉、CRUD(Create, Read, Update, Delete)
"""

# =============================================================================
# sqlite3 為聯繫 SQLite 的資料庫 (database) 模組 (module) ，基本使用順序為
# 1. 用 connect() 函數建立跟資料庫檔案聯繫的 Connection 物件。
# 2. 由 Connection 物件的 cursor() 方法建立 Cursor 物件。
# 3. 利用 Cursor 物件的 execute() 方法進行資料庫操作。
# 4. 利用 Connection 物件的 commit() 方法做資料庫檔案更新。
# 5. 利用 Connection 物件的 close() 方法做關閉跟資料庫檔案的聯繫。
# =============================================================================

import sqlite3
import pandas as pd
import glob

# 連接 stocks.sqlite 資料庫
conn = sqlite3.connect('stocks.sqlite')

# 傳回 cursor 物件
cursor = conn.cursor()

# =============================================================================
#  建立名為 TransactionInfo 的資料表
#  欄位有：日期(Date)、股票代號(Symbol)、股票名稱(Name)、開盤價(Open)、最高價(High)、最低價(Low)、收盤價(Close)、成交股數(Volume)、成交金額(Value)
#  欄位型態分別為：TEXT、TEXT、TEXT、REAL、REAL、REAL、REAL、INTEGER、INTEGER
# =============================================================================
#TransactionTableSQL = 'CREATE TABLE IF NOT EXISTS TransactionInfo(Date TEXT NOT NULL, Symbol TEXT NOT NULL, Name TEXT NOT NULL, Open REAL NOT NULL, High REAL NOT NULL, Low REAL NOT NULL, Close REAL NOT NULL, Volume INTEGER NOT NULL, Value INTEGER NOT NULL)'
#cursor.execute(TransactionTableSQL)

# =============================================================================
#  新增一筆紀錄
#  SQL 指令格式
#  INSERT INTO table_name(field1, field2,..., fieldn) VALUES(value1, value2,...,valuen)
#  缺漏的欄位將被填入 NULL 值, 但若缺漏的欄位型態為 NOT NULL 將產生執行錯誤
# =============================================================================
#RecordSQL = 'INSERT INTO TransactionInfo(Date, Symbol, Name, Open, High, Low, Close, Volume, Value) VALUES(' + str(i) + ', "台積電", "200", "220", "180", "200", "10000000000000000", "1050000000000000000000")'
#cursor.execute(dateSQL)
#conn.commit()


for file in glob.glob('2019*臺股開高低收量額.csv'):
    df = pd.read_csv(file, encoding='utf-8', sep=',', thousands=',')
    df['日期'] = file[:10]
    print('正在處理', file[:10], '的資料')
    df = df[['日期', '證券代號', '證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數', '成交金額']]
    for i in df.index:
        SQL = 'INSERT INTO TransactionInfo(Date, Symbol, Name, Open, High, Low, Close, Volume, Value) VALUES(\"' + str(df["日期"][i]) + '\",\"' + str(df["證券代號"][i]) + '\",\"' + str(df["證券名稱"][i]) + '\",\"' + str(df["開盤價"][i]) + '\",\"' + str(df["最高價"][i]) + '\",\"' + str(df["最低價"][i]) + '\",\"' + str(df["收盤價"][i]) + '\",\"' + str(df["成交股數"][i]) + '\",\"' + str(df["成交金額"][i]) + '\")'
        cursor.execute(SQL)
    conn.commit()
conn.close()