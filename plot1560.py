# -*- coding: utf-8 -*-
"""
Created on Sun Aug 26 15:58:01 2018

@author: Oscar
"""

import matplotlib.pyplot as plt
import pandas as pd
import numpy as np

f_open = open( '1560的開高低收量本.csv' , encoding = 'utf-8' )
ohlcv_1560 = pd.read_csv( f_open , encoding = 'utf-8' , thousands = ',')

plt.style.use('seaborn-whitegrid')
fig = plt.figure()
ax = plt.axes()

#%%
plt.plot(ohlcv_1560.index[-100:] , ohlcv_1560['收盤價'][-100:] , '-' , color='blue')
plt.plot(ohlcv_1560.index[-100:] , ohlcv_1560['開盤價'][-100:] , '-' , color='red')
#%%