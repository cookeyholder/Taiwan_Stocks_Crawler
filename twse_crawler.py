import datetime
import glob
import random
import sqlite3
import time
from io import StringIO

import pandas as pd
import requests


def is_warrant(code):
    """
    從代號判斷是不是權證
    權證命名規則請看 https://www.twse.com.tw/zh/page/products/securities/warrant/faq.html
    """
    if len(code) == 6 and code[-1] in ["P", "F", "Q", "C", "B", "X", "Y"]:
        if int(code[:-1]) >= 3001 and int(code[:-1]) <= 8999:
            return True
        else:
            return False
    elif (
        len(code) == 6 and code.isdigit() and int(code) >= 30001 and int(code) <= 89999
    ):
        return True
    else:
        return False


def twse_crawler(crawl_date):
    twse_url = (
        "http://www.twse.com.tw/exchangeReport/MI_INDEX?response=csv&date="
        + crawl_date.strftime("%Y%m%d")
        + "&type=ALL"
    )
    r = requests.get(twse_url)

    # 第 275 列到倒數第 2 列才是所要的資料
    lines = r.text.split("\n")[275:-2]
    stocks = [lines.pop(0).replace("=", "").replace('"', "").split(",")]

    for line in lines:
        data = line.split('","')
        data[0] = data[0].replace("=", "")
        data[0] = data[0].replace('"', "")
        # 如果不是權證
        if not is_warrant(data[0]):
            data = [i.replace('"', "") for i in data]
            data = [i.replace(",", "") for i in data]
            data = [i.replace("--", "0") for i in data]
            data = [i.strip() for i in data]
            stocks.append(data)
    twse_stocks = pd.DataFrame(stocks[1:], columns=stocks[0])
    twse_stocks = twse_stocks.apply(pd.to_numeric, errors="ignore")

    return twse_stocks


if __name__ == "__main__":
    date_of = datetime.datetime(2022, 12, 9)
    df = twse_crawler(date_of)
    df.to_csv(
        f"{date_of.strftime("%Y-%m-%d")}上市公司股票成交資訊.csv", encoding="utf-8"
    )
    print(f"已輸出到{date_of.strftime("%Y-%m-%d")}上市公司股票成交資訊.csv")
