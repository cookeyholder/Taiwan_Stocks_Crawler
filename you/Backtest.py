import datetime
import math
import random
import re
from collections import namedtuple

# 用 namedtuple 來紀錄交易
Records = namedtuple(
    "Records", ["serial", "trade_date", "action", "target", "price", "link", "reasons"]
)


class TransactionRecord:
    """
    記帳簿類別
    屬性(attributes)：
        serial: 序號

    方法(methods)：
        add: 增加紀錄
        to_list: 轉成 list
        incomes: 計算每一筆交易的損益
    """

    def __init__(self):
        self.history = []

    def add(self, trade_date, action, target, price, reasons):
        """
        Add a record of transaction.
        """
        serial = datetime.datetime.now().strftime("%Y%m%d%H%M%S") + str(
            random.randint(100000, 999999)
        )  # 交易序號
        link = "https://goodinfo.tw/StockInfo/ShowK_Chart.asp?STOCK_ID=" + target
        self.history.append(Records(serial, trade_date, action, target, price, link, reasons))

    def to_list(self):
        """
        Turn all records into and 2D list.
        """
        pass

    def incomes(
        self,
        tax_rate=0.003,  # 證交稅稅率
        fee_rate=0.001425,  # 手續費原始費率
        discount=1,  # 手續費折扣
        min_fee=20,  # 手續費低消
        shares=1000,
    ):
        """
        Calculate the incomes of every sigle record.
        """
        incomes = {}
        # 如果是 ETF 則將證交稅率改為千分之一
        if stock_id[:2] == "00":
            tax_rate = 0.001

        turnover = round(self.serial.price * shares)  # 成交額
        tax = math.floor(turnover * tax_rate)  # 證交稅
        fee = max(math.floor(turnover * fee_rate * discount), min_fee)  # 傭金
        incomes["date"] = turnover - tax - fee
        return incomes


def backtest(targets=None, start=None, end=None, strategy=None):
    """
    targets: list of stocks
    start: start date
    end: end date
    strategy: the transact strategy which return one of Buy, Sell, Hold.
    """
    # 新增一個空白記帳簿物件
    Booking = TransactionRecord()

    try:
        # 如果沒有標的物 targets
        if targets is None:
            print("No target stock.")
        elif not targets:
            print("No target stock.")

        # 處理開始和結束的日期
        # 如果沒有指定結束日期，則以今天為結束日期
        ptn = re.compile(r"([1-9]\d{3}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01]))")

        if end is None:
            end = datetime.date.today()
            # 如果沒有指定開始日期，則以結束日期往前推 365 天當做開始日期
            if start is None:
                start = end - datetime.timedelta(days=365)
            elif ptn.match(start):
                start = datetime.datetime.strptime(start, "%y-%m-%d").date()

        elif ptn.match(end):
            end = datetime.datetime.strptime(end, "%y-%m-%d").date()
            # 如果沒有指定開始日期，則以結束日期往前推 365 天當做開始日期
            if start is None:
                start = end - datetime.timedelta(days=365)
            elif ptn.match(start):
                start = datetime.datetime.strptime(start, "%y-%m-%d").date()

    except:
        """
        可能的錯誤：
            開始、結束日期不是 YYYY-MM-DD 格式
            結束日期比開始日期還早
        """
        pass

    # 建立回測期間所有的日期序列，以資料庫中取出的日期為依據
    trade_dates = []

    # 逐日回測，並將結果存入記帳簿
    for trade_date in trade_dates:
        for target in targets:
            answer = strategy(target=target, trade_date=trade_date)
            if answer[0] == "Buy":
                pass

            if answer[0] == "Sell":
                pass

            if answer[0] == "Hold":
                continue

    return Booking
