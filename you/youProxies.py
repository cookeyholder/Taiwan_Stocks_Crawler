import csv
import datetime
import os
import random

import loguru
import pandas as pd
import requests

proxies = []


def getProxy():
    global proxies
    # 若代理清單內已無代理，則重新下載
    if len(proxies) == 0:
        crawl_proxies()
    proxy = random.choice(proxies)
    proxies.remove(proxy)
    loguru.logger.debug(f"getProxy: 還有 {len(proxies)} 個 proxy 可以用。")
    return proxy


def crawl_proxies():
    global proxies
    now = datetime.datetime.now()
    hour = f"{now:%Y%m%d%H}"
    filename = f"proxies-{hour}.csv"
    filepath = f"{filename}"
    if os.path.isfile(filepath):
        loguru.logger.info(f"crawl_proxies: {filename} exists.")
        loguru.logger.warning(f"crawl_proxies: {filename} is loading...")
        with open(filepath, newline="", encoding="utf-8-sig") as f:
            reader = csv.DictReader(f)
            for row in reader:
                proxy = row["Proxy"]
                proxies.append(proxy)
        loguru.logger.success(f"crawl_proxies: {filename} is loaded.")
    else:
        loguru.logger.info(f"crawl_proxies: {filename} does not exist.")

        # proxies = proxies + crawl_proxiesFromSSLProxy()
        proxies = proxies + crawl_proxiesFromFreeProxyList()
        proxies = proxies + crawl_proxiesFromProxyNova()
        proxies = list(dict.fromkeys(proxies))
        # 檢查 proxies 是否可正常連線
        # proxies = test_proxy_alive(proxies)

        loguru.logger.warning(f"crawl_proxies: {filename} is saving...")
        with open(filepath, "w", newline="", encoding="utf-8-sig") as f:
            writer = csv.writer(f)
            writer.writerow(["Proxy"])
            for proxy in proxies:
                writer.writerow([proxy])
        loguru.logger.success(f"crawl_proxies: {filename} is saved.")


def test_proxy_alive(proxies):
    print(f"檢查 Proxy 是否可正常連線，共有 {len(proxies)} 筆要檢查。")
    for proxy in proxies:
        testurl = f"https://{proxy}/"
        try:
            requests.get(testurl, timeout=2)
            print(f"{proxy} 可用。")
        except Exception:
            print(f"從 proxies 中移除 {proxy}")
            proxies.remove(proxy)
    print("檢查 Proxy 是否可正常連線結束")
    return proxies


def crawl_proxiesFromProxyNova():
    print("從 ProxyNova 下載 Proxie 清單")
    proxies = []

    # 按照網站規則使用各國代碼傳入網址取得各國 IP 代理
    # countries = ['tw', 'jp', 'kr', 'hk', 'us', 'id', 'my', 'th', 'vn', 'ph']
    countries = ["tw", "jp", "kr", "hk"]

    for country in countries:
        url = f"https://www.proxynova.com/proxy-server-list/country-{country}/"
        r = requests.get(url)
        if r.status_code != requests.codes.ok:
            # 繼續抓下一個國家的 proxies
            continue

        df = pd.read_html(r.text)[0].iloc[:, [0, 1]]
        mask = df["Proxy Port"].apply(lambda x: str(x).isdigit())
        df = df[mask]
        df["Proxy IP"] = df["Proxy IP"].apply(lambda x: str(x).replace("document.write('", ""))
        df["Proxy IP"] = df["Proxy IP"].apply(lambda x: str(x).replace("');", ""))

        for i in range(len(df)):
            proxies.append(df["Proxy IP"].iloc[i] + ":" + df["Proxy Port"].iloc[i])
    print("從 ProxyNova 下載 Proxie 清單結束")
    return proxies


def crawl_proxiesFromFreeProxyList():
    print("從 FreeProxyList 下載 Proxie 清單")
    proxies = []
    url = "https://free-proxy-list.net/"
    r = requests.get(url)
    df = pd.read_html(r.text)[0].iloc[:, [0, 1]].dropna()

    # 分離出 ip 和 port
    ips = df.iloc[:, 0].to_list()
    ports = df.iloc[:, 1].to_list()
    ports = [int(x) for x in ports]

    for ip, port in zip(ips, ports, strict=False):
        combine = str(ip) + ":" + str(port)
        proxies.append(combine)
    print("從 FreeProxyList 下載 Proxie 清單結束")
    return proxies


def crawl_proxiesFromSSLProxy():
    print("從 SSLProxy 下載 Proxie 清單")
    proxies = []
    url = "https://www.sslproxies.org/"
    r = requests.get(url)
    df = pd.read_html(r.text)[0].iloc[:, [0, 1]].dropna()

    # 分離出 ip 和 port
    ips = df.iloc[:, 0].to_list()
    ips = [str(x) for x in ips]
    ports = df.iloc[:, 1].to_list()
    ports = [int(x) for x in ports]
    ports = [str(x) for x in ports]

    for ip, port in zip(ips, ports, strict=False):
        combine = ip + ":" + port
        proxies.append(combine)
    print("從 SSLProxy 下載 Proxie 清單結束")
    return proxies
