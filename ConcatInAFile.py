# -*- coding: utf-8 -*-
"""
Created on Tue Dec 24 16:04:32 2019

@author: slwang
"""


from io import StringIO
import pandas as pd
import glob


for file in glob.glob('*臺股開高低收量.csv'):
    df = pd.read_csv(file, encoding='utf-8')
    df['日期'] = file[:10]
    df = df[['日期', '證券代號', '證券名稱', '開盤價', '最高價', '最低價', '收盤價', '成交股數']]
    df.to_csv('臺股股價.csv', mode='a+', encoding='utf-8', index=False, header=False)
