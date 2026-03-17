# -*- coding: utf-8 -*-
"""
Created on Tue Jan 14 19:16:01 2020

@author: Cookey
"""

import csv
from datetime import datetime
import numpy as np
import os
import pandas as pd
import sqlite3
import requests
import time


# 讀取交易資料
def ReadData(NumberofDays):
    # 讀入開盤價格
    OpenPrices = pd.read_csv(
        "臺股開盤價格.csv", encoding="utf-8", index_col="Unnamed: 0", sep=",", thousands=","
    )
    OpenPrices = OpenPrices.fillna(0.0)
    OpenPrices = OpenPrices.replace("--", 0.0)
    OpenPrices = OpenPrices[-NumberofDays:]
    OpenPrices = OpenPrices.apply(pd.to_numeric, errors="coerce")
    OpenPrices = OpenPrices.sort_index()

    # 讀入收盤價格
    ClosePrices = pd.read_csv(
        "臺股收盤價格.csv", encoding="utf-8", index_col="Unnamed: 0", sep=",", thousands=","
    )
    ClosePrices = ClosePrices.fillna(0.0)
    ClosePrices = ClosePrices.replace("--", 0.0)
    ClosePrices = ClosePrices[-NumberofDays:]
    ClosePrices = ClosePrices.apply(pd.to_numeric, errors="coerce")
    ClosePrices = ClosePrices.sort_index()

    # 讀入交易股數
    Volume = pd.read_csv("臺股成交股數.csv", encoding="utf-8", index_col="Unnamed: 0")
    Volume = Volume.fillna(0.0)
    Volume = Volume.replace("--", 0.0)
    Volume = Volume[-NumberofDays:]
    Volume = Volume.apply(pd.to_numeric, errors="coerce")
    Volume = Volume.sort_index()

    return (OpenPrices, ClosePrices, Volume)


# 到證交所抓取股票代號
def crawlStockID():
    twse_market = "https://isin.twse.com.tw/isin/C_public.jsp?strMode=2"
    otc_market = "https://isin.twse.com.tw/isin/C_public.jsp?strMode=4"

    myheader = {
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                AppleWebKit/537.36 (KHTML, like Gecko) \
                Chrome/75.0.3770.142 Safari/537.36"
    }
    stockcodes = pd.DataFrame(columns=["代號", "名稱", "上市日", "市場別", "產業別", "CFICode"])
    try:
        for market in [twse_market, otc_market]:
            r = requests.get(market, headers=myheader)
            df = pd.read_html(r.text)[0]
            df.columns = [
                "有價證券代號及名稱",
                "國際證券辨識號碼(ISIN Code)",
                "上市日",
                "市場別",
                "產業別",
                "CFICode",
                "備註",
            ]
            df = df[df["CFICode"] == "ESVUFR"].reset_index()
            df[["代號", "名稱"]] = df["有價證券代號及名稱"].str.split(expand=True)
            df.drop(["有價證券代號及名稱", "備註", "國際證券辨識號碼(ISIN Code)"], axis=1)
            df = df[["代號", "名稱", "上市日", "市場別", "產業別", "CFICode"]]
            stockcodes = pd.concat([stockcodes, df], axis=0)
    except:
        print("請檢查網路連線。")

    stockcodes.to_csv("StockID.csv", index=False)
    return stockcodes


# 取得所有上市櫃股票的代號
def GetAllStockID():
    filectime = time.localtime(os.stat("StockID.csv").st_mtime)
    today = datetime.today()
    filenotexist = not os.path.isfile("StockID.csv")
    filenotupdate = (filectime.tm_mon, filectime.tm_mday) != (today.month, today.day)

    if filenotexist:
        print("股票代號檔案不存在！先從證交所抓，再讀入")
        crawlStockID()
        df = pd.read_csv(
            "StockID.csv", encoding="utf-8", sep=",", thousands=",", dtype={"代號": str}
        )
        df = df[df["CFICode"] == "ESVUFR"]
        df = df.drop(["上市日", "市場別", "產業別", "CFICode"], axis=1)
        df = df.set_index("代號")
    elif filenotupdate:
        print("股票代號檔案太舊，從證交所更新股票代號")
        crawlStockID()
        df = pd.read_csv(
            "StockID.csv", encoding="utf-8", sep=",", thousands=",", dtype={"代號": str}
        )
        df = df[df["CFICode"] == "ESVUFR"]
        df = df.drop(["上市日", "市場別", "產業別", "CFICode"], axis=1)
        df = df.set_index("代號")
    else:
        print("讀取股票代號檔案")
        df = pd.read_csv(
            "StockID.csv", encoding="utf-8", sep=",", thousands=",", dtype={"代號": str}
        )
        df = df[df["CFICode"] == "ESVUFR"]
        df = df.drop(["上市日", "市場別", "產業別", "CFICode"], axis=1)
        df = df.set_index("代號")

    return df


# 篩選壓縮、帶量且剛突破的個股名單
def WaterBuyin(OpenPrices, ClosePrices, Volume):
    Std = pd.DataFrame(columns=ClosePrices.columns)  # 標準差
    CV = pd.DataFrame(columns=ClosePrices.columns)  # 變異係數
    UBand = pd.DataFrame(columns=ClosePrices.columns)  # 上通道
    DBand = pd.DataFrame(columns=ClosePrices.columns)  # 下通道
    MovingVolume = pd.DataFrame(columns=ClosePrices.columns)  # 前 5 日均量
    AllStockIDs = GetAllStockID()

    print("開始篩選可以買進的名單。")
    # '買進日期', '股票代號', '股票名稱', '買進價', 'Yahoo連結', '出場日期', '賣出價', '盈虧', '報酬率'
    WaterBuyinList = pd.DataFrame(
        index=[
            "買進日期",
            "股票代號",
            "股票名稱",
            "買進價",
            "Yahoo連結",
            "出場日期",
            "賣出價",
            "盈虧",
            "報酬率",
            "出場原因",
        ]
    )
    for stock_id in ClosePrices.columns:
        Std[stock_id] = ClosePrices[stock_id].rolling(20).std()
        CV[stock_id] = (
            ClosePrices[stock_id].rolling(20).std() / ClosePrices[stock_id].rolling(20).mean()
        )
        UBand[stock_id] = (
            ClosePrices[stock_id].rolling(20).mean() + 2 * ClosePrices[stock_id].rolling(20).std()
        )
        DBand[stock_id] = (
            ClosePrices[stock_id].rolling(20).mean() - 2 * ClosePrices[stock_id].rolling(20).std()
        )
        MovingVolume[stock_id] = Volume[stock_id].rolling(5).mean()

        # 設定 5 層濾網
        # 起漲，收盤價超過上通道，且收紅 K
        mask1 = (
            (ClosePrices[stock_id][-1] > UBand[stock_id][-1])
            & (ClosePrices[stock_id][-2] < UBand[stock_id][-2])
            & (ClosePrices[stock_id][-1] > OpenPrices[stock_id][-1])
        )

        # 帶量，起漲日的成交量超過前 5 日均量的三倍到十倍
        mask2 = (Volume[stock_id][-1] > 3 * MovingVolume[stock_id][-1]) & (
            Volume[stock_id][-1] < 10 * MovingVolume[stock_id][-1]
        )

        # 壓縮，布林帶寬 = 4 倍變異係數 < 0.15，改為變異係數
        SqueezeDaysLimit = 60  # 壓縮的天數
        # mask3 = (Std[stock_id][-SqueezeDaysLimit:].max() < 0.15/4)
        mask3 = CV[stock_id][-SqueezeDaysLimit:].max() < 0.15 / 4

        # 流動性，成交金額超過 1 千萬
        TransactionValue = 1000_0000  # 成交股數
        mask4 = Volume[stock_id][-1] > TransactionValue

        # 股價上限，依可操作現金為準
        # 股價下限大於 10 元
        PriceLimit = 38  # 股價限制，目前只能買 40 元以下的股票。
        # mask5 = (ClosePrices[stock_id][-1] < PriceLimit) & (ClosePrices[stock_id][-1] >= 10)
        mask5 = ClosePrices[stock_id][-1] > 10

        # 如果滿足以上所設的濾網，則將日期、代號、收盤價存入
        if mask1 and mask2 and mask3 and mask4 and mask5:
            WaterBuyinList[stock_id] = [
                list(ClosePrices.index)[-1],
                stock_id,
                AllStockIDs["名稱"][stock_id],
                ClosePrices[stock_id][-1],
                "https://tw.stock.yahoo.com/q/ta?s=" + stock_id,
                "",
                "",
                "",
                "",
                "",
            ]

    return WaterBuyinList.transpose()


# 過濾要出場的名單
def WaterSelloutList(OpenPrices, ClosePrices, Volume):
    SelloutList = pd.read_csv("阿水一式名單.csv", encoding="utf-8", keep_default_na=False)
    needcheck = SelloutList[SelloutList["出場日期"] == ""]
    AllStockIDs = GetAllStockID()

    print("開始檢查出場名單。")
    for stock_id in needcheck["股票代號"]:
        # 向下跳空
        out1 = (
            OpenPrices[str(stock_id)][-1]
            < min(OpenPrices[str(stock_id)][-2], ClosePrices[str(stock_id)][-2]) * 0.95
        )

        # 虧損超過 5%
        out2 = (
            ClosePrices[str(stock_id)][-1]
            / SelloutList[SelloutList["股票代號"] == stock_id]["買進價"].iloc[-1]
            - 1
        ) <= -0.04

        # MA5 向下
        MA5 = ClosePrices[str(stock_id)].rolling(5).mean()
        out3 = MA5[-1] < MA5[-2]

        if out1 or out2 or out3:
            SelloutList.loc[SelloutList["股票代號"] == stock_id, "出場日期"] = ClosePrices.index[-1]
            SelloutList.loc[SelloutList["股票代號"] == stock_id, "賣出價"] = ClosePrices[
                str(stock_id)
            ][-1]
            SelloutList.loc[SelloutList["股票代號"] == stock_id, "盈虧"] = (
                ClosePrices[str(stock_id)][-1]
                - SelloutList.loc[SelloutList["股票代號"] == stock_id, "買進價"]
            )
            SelloutList.loc[SelloutList["股票代號"] == stock_id, "報酬率"] = (
                SelloutList.loc[SelloutList["股票代號"] == stock_id, "盈虧"]
                / SelloutList.loc[SelloutList["股票代號"] == stock_id, "買進價"]
                - 1
            ) * 100
            if out1:
                SelloutList.loc[SelloutList["股票代號"] == stock_id, "出場原因"] = "開盤向下跳空"
            elif out2:
                SelloutList.loc[SelloutList["股票代號"] == stock_id, "出場原因"] = "虧損超過 5%"
            else:
                SelloutList.loc[SelloutList["股票代號"] == stock_id, "出場原因"] = (
                    "五日均線開始下彎"
                )

    return SelloutList


# 讀取近 200 天的交易資料
Data = ReadData(200)  # 近一年內的交易資料


# 篩選買進名單，過濾出場名單
WaterBuy = WaterBuyin(Data[0], Data[1], Data[2])
WaterBuy.to_csv("阿水一式名單.csv", mode="a+", index=False, header=False)

WaterSell = WaterSelloutList(Data[0], Data[1], Data[2])
WaterSell.to_csv("阿水一式名單.csv", mode="w+", index=False, header=True)


# 印出符合的個股名稱及 Yahoo連結
today = list(Data[0].index)[-1]
print(today, "滿足阿水一式的個股有：")
if len(WaterBuy) != 0:
    for index, row in WaterBuy[WaterBuy["買進日期"] == today].iterrows():
        print(
            row["股票名稱"],
            row["Yahoo連結"],
        )
else:
    print("沒有進場名單")

print("")
print(today, "出場的個股有：")
if len(WaterSell[WaterSell["出場日期"] == today]) != 0:
    for index, row in WaterSell[WaterSell["出場日期"] == today].iterrows():
        print(
            row["股票名稱"],
            row["Yahoo連結"],
        )
else:
    print("沒有出場名單")
