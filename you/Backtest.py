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
        stock_id="",
        price=0.0,
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

        turnover = round(float(price) * shares)  # 成交額
        tax = math.floor(turnover * tax_rate)  # 證交稅
        fee = max(math.floor(turnover * fee_rate * discount), min_fee)  # 傭金
        incomes["date"] = turnover - tax - fee
        return incomes


def _resolve_backtest_window(start, end):
    ptn = re.compile(r"([1-9]\d{3}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01]))")

    if end is None:
        end_date = datetime.date.today()
    elif isinstance(end, str) and ptn.match(end):
        end_date = datetime.datetime.strptime(end, "%Y-%m-%d").date()
    else:
        raise ValueError("end 日期格式錯誤")

    if start is None:
        start_date = end_date - datetime.timedelta(days=365)
    elif isinstance(start, str) and ptn.match(start):
        start_date = datetime.datetime.strptime(start, "%Y-%m-%d").date()
    else:
        raise ValueError("start 日期格式錯誤")

    if start_date > end_date:
        raise ValueError("start 不可晚於 end")

    return start_date, end_date


def backtest(targets=None, start=None, end=None, strategy=None):
    """
    targets: list of stocks
    start: start date
    end: end date
    strategy: the transact strategy which return one of Buy, Sell, Hold.
    """
    # 新增一個空白記帳簿物件
    Booking = TransactionRecord()

    if not targets:
        print("No target stock.")
        return Booking

    if strategy is None:
        return Booking

    try:
        _resolve_backtest_window(start, end)
    except Exception:
        return Booking

    # 建立回測期間所有的日期序列，以資料庫中取出的日期為依據
    trade_dates = []

    # 逐日回測，並將結果存入記帳簿
    for trade_date in trade_dates:
        for target in targets:
            answer = strategy(target=target, trade_date=trade_date)
            action = answer[0] if answer else "Hold"
            if action == "Hold":
                continue
            if action in {"Buy", "Sell"}:
                pass

    return Booking
