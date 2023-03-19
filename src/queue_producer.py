#!/usr/bin/env python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters(host='192.168.10.200', port=30672))
channel = connection.channel()

channel.queue_declare(queue='chess-files', durable=True)

months = [(year, month) for year in range(2013, 2023) for month in range(1, 13)]
months += [(2023, 1), (2023, 2)]

for year, month in reversed(months):
    filename = f'/home/max/storage/chess/lichess_db_standard_rated_{year}-{month:02d}.pgn.zst'
    channel.basic_publish(exchange='', routing_key='chess-files', body=filename)
    print(f" [x] Sent '{filename}'")

connection.close()