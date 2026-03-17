#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Created on Fri Feb 14 03:00:45 2020

@author: cookey
"""


import csv
import datetime
import os
import random
import re
import time
import urllib.parse

import js2py
import json
import loguru
import pyquery
import requests
from bs4 import BeautifulSoup

now = datetime.datetime.now()
proxies = []

# 隨機取出一組代理
def getProxy():
    global proxies
    # 若代理清單內已無代理，則重新下載
    if len(proxies) == 0:
        getProxies()
    proxy = random.choice(proxies)
    loguru.logger.debug(f'getProxy: 取得代理伺服器 {proxy}')
    proxies.remove(proxy)
#    loguru.logger.debug(f'getProxy: {len(proxies)} proxies is unused.')
    loguru.logger.debug(f'getProxy: 還有 {len(proxies)} 個 proxy 可以用。')
    return proxy

def reqProxies(hour):
    global proxies
    proxies = proxies + getProxiesFromSSLProxy()
    proxies = proxies + getProxiesFromFreeProxyList()
    proxies = proxies + getProxiesFromProxyNova()
    proxies = proxies + getProxiesFromGatherProxy()
    proxies = list(dict.fromkeys(proxies))
#    loguru.logger.debug(f'reqProxies: {len(proxies)} proxies is found.')
    loguru.logger.debug(f'reqProxies: 共有 {len(proxies)} 個 proxy。')

def getProxies():
    global proxies
    now = datetime.datetime.now()
    hour = f'{now:%Y%m%d%H}'
    filename = f'proxies-{hour}.csv'
    filepath = f'{filename}'
    if os.path.isfile(filepath):
        loguru.logger.info(f'getProxies: {filename} exists.')
        loguru.logger.warning(f'getProxies: {filename} is loading...')
        with open(filepath, 'r', newline='', encoding='utf-8-sig') as f:
            reader = csv.DictReader(f)
            for row in reader:
                proxy = row['Proxy']
                proxies.append(proxy)
        loguru.logger.success(f'getProxies: {filename} is loaded.')
    else:
        loguru.logger.info(f'getProxies: {filename} does not exist.')
        reqProxies(hour)
        loguru.logger.warning(f'getProxies: {filename} is saving...')
        with open(filepath, 'w', newline='', encoding='utf-8-sig') as f:
            writer = csv.writer(f)
            writer.writerow([
                'Proxy'
            ])
            for proxy in proxies:
                writer.writerow([
                    proxy
                ])
        loguru.logger.success(f'getProxies: {filename} is saved.')


def getProxiesFromProxyNova():
    proxies = []
    # 按照網站規則使用各國代碼傳入網址取得各國 IP 代理
    countries = [
        'tw',
        'jp',
        'kr',
        'id',
        'my',
        'th',
        'vn',
        'ph',
        'hk',
        'uk',
        'us'
    ]
    for country in countries:
        url = f'https://www.proxynova.com/proxy-server-list/country-{country}/'
        loguru.logger.debug(f'getProxiesFromProxyNova: {url}')
        loguru.logger.warning(f'getProxiesFromProxyNova: downloading...')
        response = requests.get(url)
        if response.status_code != 200:
            loguru.logger.debug(f'getProxiesFromProxyNova: status code is not 200')
            continue
        loguru.logger.success(f'getProxiesFromProxyNova: downloaded.')
        d = pyquery.PyQuery(response.text)
        table = d('table#tbl_proxy_list')
        rows = list(table('tbody:first > tr').items())
        loguru.logger.warning(f'getProxiesFromProxyNova: scanning...')
        for row in rows:
            tds = list(row('td').items())
            # 若為分隔行則僅有 1 格
            if len(tds) == 1:
                continue
            # 取出 IP 欄位內的 JavaScript 程式碼
            js = row('td:nth-child(1) > abbr').text()
            # 去除 JavaScript 程式碼開頭的 document.write( 字串與結尾的 ); 字串，
            # 再與可供 js2py 執行後回傳指定變數的 JavaScript 程式碼相結合
            js = 'let x = %s; x' % (js[15:-2])
            # 透過 js2py 執行取得還原後的 IP
            ip = js2py.eval_js(js).strip()
            # 取出 Port 欄位值
            port = row('td:nth-child(2)').text().strip()
            # 組合 IP 代理
            proxy = f'{ip}:{port}'
            proxies.append(proxy)
        loguru.logger.success(f'getProxiesFromProxyNova: scanned.')
        loguru.logger.debug(f'getProxiesFromProxyNova: {len(proxies)} proxies is found.')
        # 每取得一個國家代理清單就休息一秒，避免頻繁存取導致代理清單網站封鎖
        time.sleep(1)
    return proxies


def getProxiesFromFreeProxyList():
    proxies = []
    url = 'https://free-proxy-list.net/'
    loguru.logger.debug(f'getProxiesFromFreeProxyList: {url}')
    loguru.logger.warning(f'getProxiesFromFreeProxyList: downloading...')
    response = requests.get(url)
    if response.status_code != 200:
        loguru.logger.debug(f'getProxiesFromFreeProxyList: status code is not 200')
        return
    loguru.logger.success(f'getProxiesFromFreeProxyList: downloaded.')
    d = pyquery.PyQuery(response.text)
    trs = list(d('table#proxylisttable > tbody > tr').items())
    loguru.logger.warning(f'getProxiesFromFreeProxyList: scanning...')
    for tr in trs:
        # 取出所有資料格
        tds = list(tr('td').items())
        # 取出 IP 欄位值
        ip = tds[0].text().strip()
        # 取出 Port 欄位值
        port = tds[1].text().strip()
        # 組合 IP 代理
        proxy = f'{ip}:{port}'
        proxies.append(proxy)
    loguru.logger.success(f'getProxiesFromFreeProxyList: scanned.')
    loguru.logger.debug(f'getProxiesFromFreeProxyList: {len(proxies)} proxies is found.')
    return proxies


def getProxiesFromGatherProxy():
    proxies = []
    # 按照網站規則使用各國國名傳入網址取得各國 IP 代理
    countries = [
        'Taiwan',
        'Japan',
        'United States',
        'Thailand',
        'Vietnam',
        'Indonesia',
        'Singapore',
        'Philippines',
        'Malaysia',
        'Hong Kong'
    ]
    for country in countries:
        url = f'http://www.gatherproxy.com/proxylist/country/?c={urllib.parse.quote(country)}'
        loguru.logger.debug(f'getProxiesFromGatherProxy: {url}')
        loguru.logger.warning(f'getProxiesFromGatherProxy: downloading...')
        response = requests.get(url)
        if response.status_code != 200:
            loguru.logger.debug(f'getProxiesFromGatherProxy: status code is not 200')
            continue
        loguru.logger.success(f'getProxiesFromGatherProxy: downloaded.')
        d = pyquery.PyQuery(response.text)
        scripts = list(d('table#tblproxy > script').items())
        loguru.logger.warning(f'getProxiesFromGatherProxy: scanning...')
        for script in scripts:
            # 取出 script 標簽中的 JavaScript 原始碼
            script = script.text().strip()
            # 去除 JavaScript 程式碼開頭的 gp.insertPrx( 字串與結尾的 ); 字串
            script = re.sub(r'^gp\.insertPrx\(', '', script)
            script = re.sub(r'\);$', '', script)
            # 將參數物件以 JSON 方式解析
            script = json.loads(script)
            # 取出 IP 欄位值
            ip = script['PROXY_IP'].strip()
            # 取出 Port 欄位值，並從 16 進位表示法解析為 10 進位表示法
            port = int(script['PROXY_PORT'].strip(), 16)
            # 組合 IP 代理
            proxy = f'{ip}:{port}'
            proxies.append(proxy)
        loguru.logger.success(f'getProxiesFromGatherProxy: scanned.')
        loguru.logger.debug(f'getProxiesFromGatherProxy: {len(proxies)} proxies is found.')
        # 每取得一個國家代理清單就休息一秒，避免頻繁存取導致代理清單網站封鎖
        time.sleep(1)
    return proxies


def getProxiesFromSSLProxy():
    proxies = []
    url ='https://www.sslproxies.org/'
    loguru.logger.debug(f'getProxiesFromSSLProxy: {url}')
    loguru.logger.warning(f'getProxiesFromSSLProxy: downloading...')
    response = requests.get(url)
    soup = BeautifulSoup(response.text, 'lxml')
    trs = soup.find('tbody').find_all('tr')
    for tr in trs:
        tds = tr.find_all('td')
        proxy = tds[0].text.strip() + ':' + tds[1].text.strip()
        proxies.append(proxy)

    loguru.logger.success(f'getProxiesFromSSLProxy: scanned.')
    loguru.logger.debug(f'getProxiesFromSSLProxy: {len(proxies)} proxies is found.')
    time.sleep(1)
    return proxies