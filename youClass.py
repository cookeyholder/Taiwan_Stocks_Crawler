class StocksCrawler():
    def __init__(self, market_type, trade_date):
        self.type = market_type
        self.trade_date = trade_date

    def get(self):
        if self.type =='twse':
            get = TWSE().get()
        if self.type == 'otc':
            get = OTC().get()
        return get


class TWSE(StocksCrawler):
    def __init__(self):
        self.url = 'This twse crawler.'
    
    def get(self):
        pass


class OTC(StocksCrawler):
    def __init__(self):
        self.url = 'This is otc crawler.'
    

if __name__ == "__main__":
    twse_crawler = StocksCrawler('twse', '2020-12-04')
    print(twse_crawler.get())