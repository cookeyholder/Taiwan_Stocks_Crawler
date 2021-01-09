import datetime
import math

from youBanks import youBanks


class BackTest():
    def __init__(self, target=None, start_date=None, end_date=None, strategy=None, price=None):
        """
        targets: list of stocks
        start: start date
        end: end date
        strategy: the transact strategy which return one of Buy, Sell, Hold.
        """

        self.booking = youBanks()

        if strategy:
            self.strategy = strategy
        else:
            print('You need to input a strategy function!')

        if target is None:
            print('No target stock.')
            raise ValueError
        else:
            self.target = target

        if price is None:
            self.price = 'close'

        if end_date is None:
            self.end_date = datetime.date.today().strftime('%Y-%m-%d')

        if start_date is None:
            self.start_date = end_date - datetime.timedelta(days=365)

    def to_list(self):
        """
        Turn all records into and 2D list.
        """
        pass

    def incomes(
            self,
            TAX_RATE=0.003,  # 證交稅稅率
            FEE_RATE=0.001425,  # 手續費原始費率
            DISCOUNT_RATE=1,  # 手續費折扣
            MIN_FEE=20,  # 手續費低消
            SHARES=1000):
        """
        Calculate the incomes of every sigle record.
        """
        incomes = {}
        # 如果是 ETF 則將證交稅率改為千分之一
        if self.target[:2] == '00':
            TAX_RATE = 0.001

        # for transaction in self.booking:
            # turnover = round(self.serial.price * SHARES)  # 成交額
            # tax = math.floor(turnover * TAX_RATE)  # 證交稅
            # fee = max(math.floor(turnover*FEE_RATE*DISCOUNT_RATE), MIN_FEE)  # 傭金
            # incomes['date'] = turnover - tax - fee
        # return incomes

    def backtest(self, start=None, end=None):
        """
        targets: list of stocks
        start: start date
        end: end date
        strategy: the transact strategy which return one of Buy, Sell, Hold.
        """
        # 新增一個空白記帳簿物件
        Booking = TransactionRecord()

        try:
            # 處理開始和結束的日期
            # 如果沒有指定結束日期，則以今天為結束日期
            ptn = re.compile(
                r'([1-9]\d{3}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01]))')

            if end is None:
                end = datetime.date.today()
                # 如果沒有指定開始日期，則以結束日期往前推 365 天當做開始日期
                if start is None:
                    start = end - datetime.timedelta(days=365)
                elif ptn.match(start):
                    start = datetime.datetime.strptime(
                        start, '%y-%m-%d').date()

            elif ptn.match(end):
                end = datetime.datetime.strptime(end, '%y-%m-%d').date()
                # 如果沒有指定開始日期，則以結束日期往前推 365 天當做開始日期
                if start is None:
                    start = end - datetime.timedelta(days=365)
                elif ptn.match(start):
                    start = datetime.datetime.strptime(
                        start, '%y-%m-%d').date()

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
            answer = self.strategy(target=self.target, trade_date=trade_date)
            if answer[0] == 'Buy':
                pass

            if answer[0] == 'Sell':
                pass

            if answer[0] == 'Hold':
                continue

        return Booking

    def max_drawdown(self):
        pass
