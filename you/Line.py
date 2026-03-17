import requests

def lineNotifyMessage(token, msg):
    headers = {
        "Authorization": "Bearer " + token, 
        "Content-Type" : "application/x-www-form-urlencoded"
    }

    payload = {'message': msg}
    r = requests.post("https://notify-api.line.me/api/notify", headers=headers, params=payload)
    return r.status_code


def Vanessa(msg):
    # Vanessa token
    Vanessa_token = 'LMdb0htDvZOLd9MUCZz31iwCTCme7H7VMgxeW0UFxSY'

    headers = {
        "Authorization": "Bearer " + Vanessa_token, 
        "Content-Type" : "application/x-www-form-urlencoded"
    }

    payload = {'message': msg}
    r = requests.post("https://notify-api.line.me/api/notify", headers=headers, params=payload)
    return r.status_code


def Money(msg):
    # 阿財 token
    Money_token = 'BUQDH5qA4hmxNwev6zQUjR1bsLOg1hOFIVbfpgbZmjU'

    headers = {
        "Authorization": "Bearer " + Money_token, 
        "Content-Type" : "application/x-www-form-urlencoded"
    }

    payload = {'message': msg}
    r = requests.post("https://notify-api.line.me/api/notify", headers=headers, params=payload)
    return r.status_code
