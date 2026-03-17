# -*- coding: utf-8 -*-
"""
Created on Mon Feb 10 02:29:10 2020

@author: Cookey
"""

import pandas as pd
import sqlite3

conn = sqlite3.connect('stocks.sqlite')
cursor = conn.cursor()
#sql = 'SELECT * FROM TransactionInfo'
#df = pd.read_sql(sql, conn)
#df.to_sql('OHLCV', conn, if_exists='append', index=False)
conn.commit()
conn.close()