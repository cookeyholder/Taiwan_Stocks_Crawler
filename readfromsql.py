# -*- coding: utf-8 -*-
"""
Created on Tue Dec 31 12:52:46 2019

@author: slwang
"""

import sqlite3
import pandas as pd
import time

sTime = time.time()
conn = sqlite3.connect('stocks.sqlite')
cursor = conn.cursor()

day = '2019-01-01'
stock_id = '1101'
sql = 'SELECT * FROM (SELECT * FROM TransactionInfo WHERE Symbol = ' + stock_id + ' AND Date > ' + day + ' ORDER BY Date DESC LIMIT 60) ORDER BY Date'
df = pd.read_sql(sql, conn)

conn.close()
eTime = time.time()
print('花了 %d 秒', eTime-sTime)