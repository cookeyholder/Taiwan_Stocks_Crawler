import datetime
import random
import time

import pandas as pd
import requests
import sqlite3

import Proxies
from fake_useragent import UserAgent

def twse_crawler(crawl_date):
    proxy_ip = None
    while True:
        # 取得 proxy 的 ip
        if proxy_ip is None:
            proxy_ip = Proxies.getProxy()

        try:
            url = ''
            my_header = {'user-agent': UserAgent().random}
            r = requests.get(
                    url,
                    headers=my_header,
                    timeout=5,
                    proxies={'https':f'https://{proxy_ip}'}
                    )
            
            # 如果 requests 沒有成功連線，則將 proxy_ip 設為 None 後再重跑迴圈
            if r.status_code != requests.codes.ok:
                proxy = None
                continue
            
            # 如果成功連線就會繼續以下的程式碼
            r.encoding = 'big5-hkscs'

        # 錯誤控制：發生以下各種例外時，清除代理資訊，繼續下個迴圈
        # 連接錯誤
        except requests.exceptions.ConnectionError:
            proxy = None
            continue
        
        # 逾時錯誤
        except requests.exceptions.ConnectTimeout:
            proxy = None
            continue
        
        except requests.exceptions.ProxyError:
            proxy = None
            continue
        
        except requests.exceptions.SSLError:
            proxy = None
            continue
        
        except Exception as e:
            proxy = None
            continue
        
        # 跳出 while 迴圈
        break
    return twse_stocks
