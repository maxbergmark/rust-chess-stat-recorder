#!/usr/bin/env python
import pika

connection = pika.BlockingConnection(
    pika.ConnectionParameters(host='192.168.10.200', port=30672))
channel = connection.channel()

channel.queue_declare(queue='chess-files', durable=True)

for year in range(2014, 2024):
    for month in range(1, 13):
        filename = f'/home/max/storage/chess/lichess_db_standard_rated_{year}-{month:02d}.pgn.zst'
        channel.basic_publish(exchange='', routing_key='chess-files', body=filename)
        print(f" [x] Sent '{filename}'")

connection.close()