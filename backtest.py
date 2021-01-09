import datetime
from youBanks import youBanks


class BackTest():
    def __init__(self, start_date=None, end_date=None, strategy, price=None):
        self.strategy = strategy
        self.booking = youBanks()
        if price is None:
            self.price = close
        if end_date is None:
            self.end_date = datetime.date.today()
        if start_date is None:
            self.start_date = end_date - datetime.timedelta(days=365)

    def booking(self):
        pass

    def max_drawdown(self):
        pass
