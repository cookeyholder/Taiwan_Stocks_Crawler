# -*- coding: utf-8 -*-
"""
Created on Tue Dec 31 17:13:46 2019

@author: slwang
"""

import sqlite3
import pandas as pd


conn = sqlite3.connect('test.sqlite')
cursor = conn.cursor()

df = pd.DataFrame({'A': [1,2,3], 'B': [4,5,6]})

df.to_sql('test.sqlite', conn, if_exists='append', index=False)

conn.close()