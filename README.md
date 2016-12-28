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
[
  {
    "timetable_time": "17:33",
    "route": "539系統",
    "track": "不明",
    "destination": "時任牧場前",
    "location_stops_url": "http://hakobus.jp/s_route.php?diamasterkey=10000175&stopmasterkey_f=3&stopmasterkey_t=155&no_f=0&no_t=15#3",
    "bus": "標準のバス",
    "information": "約6分後に到着します",
    "timetable_url": "http://hakobus.jp/s_timetable_wd.php?s=11003&spm=14&d=9&generationcode=20161116"
  },
  {
    "timetable_time": "17:40",
    "route": "6333ループ121435",
    "track": "不明",
    "destination": "時任為基記念病院前・亀田市役所前方面",
    "location_stops_url": "http://hakobus.jp/s_route.php?diamasterkey=10001282&stopmasterkey_f=3&stopmasterkey_t=155&no_f=0&no_t=24#3",
    "bus": "ノンステップ",
    "information": "定刻発車の予定",
    "timetable_url": "http://hakobus.jp/s_timetable_wd.php?s=11003&spm=15&d=9&generationcode=20161116"
  },
  {
    "timetable_time": "19:03",
    "route": "9系統（時任大経由）",
    "track": "不明",
    "destination": "時任大付属中学校前",
    "location_stops_url": "http://hakobus.jp/s_route.php?diamasterkey=10000160&stopmasterkey_f=3&stopmasterkey_t=155&no_f=0&no_t=15#3",
    "bus": "ノンステップ",
    "information": "不明",
    "timetable_url": "http://hakobus.jp/s_timetable_wd.php?s=11003&spm=14&d=9&generationcode=20161116"
  }
]
```

## LICENSE
Copyright (c) 2016 Ryo Sugimoto  
See LICENSE
