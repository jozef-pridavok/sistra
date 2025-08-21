import pandas as pd
import ccxt
import json
from datetime import datetime

# BTC => bitstamp BTC/USD
# SOL => binance SOL/USDT

symbol = 'SOL'
exchange = ccxt.binance()
fiat = 'USDT'

def fetch_data(symbol, timeframe='1d', since='2010-01-01T00:00:00Z'):
    since_ms = exchange.parse8601(since)
    all_ohlcv = []
    # fetch until now
    end_ms = exchange.milliseconds()

    while since_ms < end_ms:
        # Binance default limit is 500 candles; specify limit=1000 if you need
        batch = exchange.fetch_ohlcv(symbol + '/' + fiat, timeframe, since_ms)
        if not batch:
            break
        all_ohlcv.extend(batch)
        # move pointer forward
        since_ms = batch[-1][0] + 1

    # build DataFrame
    df = pd.DataFrame(all_ohlcv, columns=['timestamp','open','high','low','close','volume'])
    df['datetime'] = pd.to_datetime(df['timestamp'], unit='ms')
    df.set_index('datetime', inplace=True)
    return df

def save_to_json(df, filename):
    # format: list of {"YYYYMMDD": close}
    records = []
    for dt, row in df.iterrows():
        key = dt.strftime('%Y%m%d')
        value = float(row['close'])
        records.append({key: value})
    with open(filename, 'w') as f:
        json.dump(records, f, indent=2)
    print(f"Data uložené do {filename}")



if __name__ == '__main__':
    filename = 'data_' + symbol.lower() + '.json'
    df = fetch_data(symbol)
    save_to_json(df, filename)
    
