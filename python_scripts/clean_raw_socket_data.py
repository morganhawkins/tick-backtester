from dataclasses import dataclass
import json
from datetime import datetime
import sys

import pandas as pd
import numpy as np



read_path = sys.argv[1]
write_path = sys.argv[2]

# read file lines and then transform into dict
lines: list[str]
with open(read_path, 'r') as file:
    lines = file.readlines()
records = list(map(json.loads, lines))

def transform_delta_record(record: dict) -> dict:
    assert(record["type"] == "orderbook_delta")
    message = record["msg"]
    ts_string = message["ts"]
    ts_seconds = datetime.fromisoformat(ts_string).timestamp()
    price=message["price"]
    yes_price = price if message["side"] == "yes" else 100-price
    side = "buy" if message["side"] == "yes" else "sell"
    return {
        "type":record["type"],
        "seq":record["seq"],
        "price":yes_price,
        "quantity":message["delta"],
        "ts":ts_seconds,
        "side":side,
    }

def transform_trade_record(record: dict) -> dict:
    assert(record["type"] == "trade")
    message = record["msg"]
    ts_string = message["ts"]
    ts_seconds = float(ts_string)
    yes_price = message["yes_price"]
    side = "buy" if message["taker_side"] == "yes" else "sell"
    return { 
        "type":record["type"],
        "seq":record["seq"],
        "price":yes_price,
        "quantity":message["count"],
        "ts":ts_seconds,
        "side":side,
    }

def transform_snapshot(snapshot: dict) -> dict:
    message = snapshot["msg"]
    buy = message["yes"]
    sell = [ [100-price, quant] for price, quant in message["no"]]
    return {
        "type":"orderbook_snapshot",
        "buy":buy, 
        "sell":sell,
    }

def transform(line: dict):
    if line["type"]=="orderbook_snapshot":
        return transform_snapshot(line)

    elif line["type"]=="orderbook_delta":
        return transform_delta_record(line)

    elif line["type"]=="trade":
        return transform_trade_record(line)
    
    else:
        raise Exception(line["type"])
    
# cleaning up timestamps and unformifying
records = list(map(transform, records))

last_record = None
next_record = None
for i, rec in enumerate(records):
    if i<=1: continue # skip snapshot

    # update prior and next record
    last_record = records[i-1] if i>0 else None
    next_record = records[i+1] if i<(len(records)-1) else None
    
    # fixing timestamps of trades order
    # needed because trades are timestamped at whole seconds no partial
    if last_record and next_record and rec["type"]=="trade":
        last_ts = last_record["ts"]
        next_ts = next_record["ts"]
        curr_ts = rec["ts"]
        if curr_ts < last_ts or curr_ts > next_ts:
            rec["ts"] = max(last_ts, next_ts)

    # finding the correspodning orderbook_delta for every trade and
    # zeroing out it's quantity
    # checking in order of
    if rec["type"]=="trade":

        match_index_delta = 0
        while True:
            # determine if the search position relative to the trade's 
            # index can be increased and/or decreased
            cant_decrease = (i-abs(match_index_delta)-1) < 0
            cant_increase = (i+abs(match_index_delta)+1) >= len(records)

            # if can't increase or decrease any more, 
            # we have checked all values
            if cant_increase and cant_decrease:
                print(f"NO MATCH FOUND OBO for trade at {i}")
                break
            # shortcut to prevent excessive runtime on poor data
            # if we don't find a match within 150 places of the trade
            # then we assume it doesn't exist
            elif abs(match_index_delta) > 150:
                print(f"NO MATCH FOUND OBO for trade at {i}")
                break
            # if we can't decrease the relative position then move in pos direction
            elif cant_decrease:
                match_index_delta = abs(match_index_delta) + 1
            # if we can't increase the relative position then move in neg direction
            elif cant_increase:
                match_index_delta = -abs(match_index_delta) - 1
            # otherwise, oscillate between checking higher and lower in increasing
            # magnitudes
            elif match_index_delta>0:
                match_index_delta *= -1 # if positive, check the negative pos
            elif match_index_delta<=0:
                match_index_delta *= -1 # if neg then we jsut came from the pos
                match_index_delta += 1 # so check one higher than the pos
            else:
                print("UNHANDLES CASE")

            # double check our index we're accessing is valid -- not necesary
            if i+match_index_delta < 0 or i+match_index_delta >= len(records):
                continue
            
            # grab record to check if it's a match
            match_record = records[i+match_index_delta]
            if (match_record["type"]=="orderbook_delta") and (-match_record["quantity"] == rec["quantity"]) and (match_record["side"] != rec["side"]):
                # it is a match if it's an orderbook_delta update AND
                # it's of the same quantity AND it has the opposite
                # side to the taker of the trade AND
                # there is no closer record that matches all conditions
                match_record["quantity"] = 0
                break

# writing results to write_path
with open(write_path, 'w') as file:
    for rec in records:
        json_str = json.dumps(rec)
        file.write(json_str+"\n")