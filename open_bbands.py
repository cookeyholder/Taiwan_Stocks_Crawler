# -*- coding: utf-8 -*-
"""
Created on Wed Feb 26 16:08:19 2020

@author: slwang
"""

import sqlite3
import datetime
import pandas as pd

dbfile = 'stocks.sqlite'
conn = sqlite3.connect(dbfile)
date_after = datetime.date.today() - datetime.timedelta(365)
read240_sql = f'SELECT * FROM OHLCV WHERE Date>"{date_after}";'

df = pd.read_sql(read240_sql, conn, index_col='Date')
stock_ids = ['5015']
for stock_id in stock_ids:
    prices = df[df['Symbol'] == stock_id][['Symbol', 'Open', 'prices']]
    prices['UB'] = prices['prices'].rolling(20).mean() + 2*prices['prices'].rolling(20).std()
    prices['BW'] = 4*prices['prices'].rolling(20).std()/(prices['prices'].rolling(20).mean() - 2*prices['prices'].rolling(20).std())
