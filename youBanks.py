import datetime
import time
import random
from itertools import count


class TransactionRecords:
    _ids = count(0)

    def __init__(self, stock_no: str):
        yahoo_prefix = 'https://tw.stock.yahoo.com/q/bc?s='
        goodinfo_prefix = 'https://goodinfo.tw/StockInfo/StockDetail.asp?STOCK_ID='
        self.serial = datetime.datetime.now().strftime('%Y%m%d%H%M%S') + \
            str(next(self._ids)).zfill(6)
        self.stock_no = stock_no
        self.stock_name = ''
        self.buy_date = ''
        self.buy_price = ''
        self.buy_amount = ''
        self.sell_date = ''
        self.sell_price = ''
        self.sell_amount = ''
        self.sell_reasons = []
        self.yahoo_link = yahoo_prefix + self.stock_no
        self.goodinfo_link = goodinfo_prefix + self.stock_no


class youBanks:
    def __init__(self):
        self.balance = []

    def show_balance(self) -> None:
        '''
        秀出所有的股票代碼、名稱及數量
        '''
        for element in self.balance:
            print(element)

    def withdraw(self, record: TransactionRecords) -> None:
        '''
        當賣出股票時，提領股票
        '''
        if isinstance(record, TransactionRecords):
            self.balance.append(record)
        pass

    def deposit(self, record: TransactionRecords) -> None:
        pass

    def to_csv(self, filename: str) -> None:
        pass
