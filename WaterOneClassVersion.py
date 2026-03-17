# -*- coding: utf-8 -*-
"""
Created on Tue Feb 11 11:06:52 2020

@author: slwang
"""


import pandas as pd
import youClass as you


def findBuy(Data):
    WaterBuyinList = pd.DataFrame(index=['買進日期',
                                         '股票代號',
                                         '股票名稱',
                                         '買進價',
                                         'Yahoo連結',
                                         '出場日期',
                                         '賣出價',
                                         '盈虧',
                                         '報酬率',
                                         '出場原因'])
    for stock_id in Data.ClosePrices.columns:
        # 設定 5 層濾網
        # 起漲，收盤價超過上通道，且收紅 K
        mask1 = (Data.ClosePrices[stock_id][-1] > Data.Uband[stock_id][-1]) & (Data.ClosePrices[stock_id][-2] < Data.Uband[stock_id][-2]) & (Data.ClosePrices[stock_id][-1] > Data.OpenPrices[stock_id][-1])

        # 帶量，起漲日的成交量超過前 5 日均量的三倍到十倍
        mask2 = (Data.Volume[stock_id][-1] > 3 * Data.MovingVolume[stock_id][-1]) & (Data.Volume[stock_id][-1] < 10 * Data.MovingVolume[stock_id][-1])

        # 壓縮，布林帶寬 = 4 倍變異係數 < 0.15，改為變異係數
        SqueezeDaysLimit = 60  # 壓縮的天數
        mask3 = (Data.Cv[stock_id][-SqueezeDaysLimit:].max() < 0.15/4)

        # 流動性，成交金額超過 1 千萬
        TransactionValue = 1000000  # 成交股數
        mask4 = (Data.Volume[stock_id][-1] > TransactionValue)

        # 股價上限，依可操作現金為準
        # 股價下限大於 10 元
        PriceLimit = 38  # 股價限制，目前只能買 40 元以下的股票。
        mask5 = (Data.ClosePrices[stock_id][-1] > 10)

        # 如果滿足以上所設的濾網，則將日期、代號、收盤價存入
        if mask1 and mask2 and mask3 and mask4 and mask5:
            WaterBuyinList[stock_id] = [list(Data.ClosePrices.index)[-1],
                                        stock_id,
                                        Data.StockIds['名稱'][stock_id],
                                        Data.ClosePrices[stock_id][-1],
                                        'https://tw.stock.yahoo.com/q/ta?s=' + stock_id,
                                        '',
                                        '',
                                        '',
                                        '',
                                        '']

    return WaterBuyinList.transpose()


waterData = you.DataSources()
waterBuy = findBuy(waterData)
waterBuy.to_csv('阿水一式名單.csv', mode='a+', index=False, header=False)
