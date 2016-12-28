# Saison
函館バス「BUSLOCATION」のバス接近情報ウェブページをスクレイピングし、そのデータをJSON形式でWebAPIのように提供することのできるツールです。

## 使い方
1. [Rust](https://www.rust-lang.org) をインストール
2. 下記を実行
```sh
% git clone https://github.com/sugryo/saison
% cd saison
% cargo build --release
% target/release/saison
```

## WebAPI の仕様
### 出発停留所から到着停留所までのバス運行情報を取得
```sh
GET /locations/:departure_stop/to/:arrived_stop
```
- departure_stop
  - 出発停留所
  - Example: 函館駅前
- arrived_stop
  - 到着停留所
  - Example: 千代台

#### レスポンス
``` Status: 200 Ok ```
```json
{"hello": "matz"}
```

## LICENSE
Copyright (c) 2016 Ryo Sugimoto  
See LICENSE
