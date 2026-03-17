import pandas as pd
import sqlite3

def test_to_db(df, db_name='test.sqlite', table_name='test'):
    '''
    將 DataFrame 輸出到資料庫，預設的資料庫名稱為 stocks.sqlite，資料表名稱為 OHLCV。
    db_name：資料庫名稱
    table_name：資料表名稱
    '''
    try:
        conn = sqlite3.connect(db_name)
        df.to_sql(table_name, conn, if_exists='append', index=False)
        conn.commit()
        conn.close()
    except Exception as e:
        print(e)
        print('寫入資料庫發生錯誤！')


def main():
    df = pd.DataFrame([[49, '阿花']], columns=['C1', 'C2'])
    print(df)
    test_to_db(df)


if __name__ == '__main__':
    main()