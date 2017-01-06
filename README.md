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
{
	"starting_stop_name": "千代台",
	"ending_stop_name": "五稜郭",
	"locations": [
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
	],
	"total": 4
}
```

### 経路情報から到着時刻を取得
```sh
GET /location?url=:locations_stops_url
```
- locations_stops_url
  - 経路情報URL
  - URL を URL エンコードしてください。
  - Example: http%3A%2F%2Fhakobus.jp%2Fs_route.php%3Fdiamasterkey%3D10000160%26stopmasterkey_f%3D3%26stopmasterkey_t%3D155%26no_f%3D0%26no_t%3D15%233

#### レスポンス
``` Status: 200 Ok ```
```json
{
  "route_name": "9系統（時任大経由）",
  "starting_stop_name": "函館駅前",
  "ending_stop_name": "時任大附属中学校前",
  "stops": [
    {
      "stop_name": "函館駅前 ５番のりば",
      "arrived_time": "17:34"
    },
    {
      "stop_name": "松風町 (1)キラリス前",
      "arrived_time": "17:38"
    },
    {
      "stop_name": "松風町 (2)電停前",
      "arrived_time": "17:41"
    },
    {
      "stop_name": "新川町 ",
      "arrived_time": "17:42"
    },
    {
      "stop_name": "千歳町 ",
      "arrived_time": "17:43"
    },
    {
      "stop_name": "昭和橋 ",
      "arrived_time": "17:45"
    },
    {
      "stop_name": "堀川町 ",
      "arrived_time": "17:47"
    },
    {
      "stop_name": "千代台 ",
      "arrived_time": "17:48"
    },
    {
      "stop_name": "中央病院前 ",
      "arrived_time": "17:52"
    },
    {
      "stop_name": "五稜郭 (1)シダックス前",
      "arrived_time": "17:54"
    },
    {
      "stop_name": "五稜郭公園入口 ",
      "arrived_time": "17:56"
    },
    {
      "stop_name": "亀田警察署前 ",
      "arrived_time": "17:57"
    },
    {
      "stop_name": "田家入口 ",
      "arrived_time": "18:00"
    },
    {
      "stop_name": "医師会病院前 ",
      "arrived_time": "18:02"
    },
    {
      "stop_name": "富岡 ",
      "arrived_time": "18:04"
    },
    {
      "stop_name": "亀田市役所前 (1)至赤川",
      "arrived_time": "18:08"
    },
    {
      "stop_name": "時任地方気象台前 ",
      "arrived_time": "18:09"
    },
    {
      "stop_name": "亀田総合振興局前 ",
      "arrived_time": "18:12"
    },
    {
      "stop_name": "美原３丁目 ",
      "arrived_time": "18:12"
    },
    {
      "stop_name": "伊勢崎長者町 ",
      "arrived_time": "18:13"
    },
    {
      "stop_name": "亀田特別支援学校前 ",
      "arrived_time": "18:14"
    },
    {
      "stop_name": "時任大附属小学校前 ",
      "arrived_time": "18:16"
    },
    {
      "stop_name": "時任台東 ",
      "arrived_time": "18:17"
    },
    {
      "stop_name": "湘南台団地入口 ",
      "arrived_time": "18:17"
    },
    {
      "stop_name": "湘南台 ",
      "arrived_time": "18:18"
    },
    {
      "stop_name": "北美原２丁目 ",
      "arrived_time": "18:19"
    },
    {
      "stop_name": "時任第一公園 ",
      "arrived_time": "18:19"
    },
    {
      "stop_name": "湘南町会館前 ",
      "arrived_time": "18:21"
    },
    {
      "stop_name": "石川町 ",
      "arrived_time": "18:21"
    },
    {
      "stop_name": "時任大学前 ",
      "arrived_time": "18:23"
    },
    {
      "stop_name": "亀田中部時任町 ",
      "arrived_time": "18:24"
    },
    {
      "stop_name": "時任為基記念病院前 ",
      "arrived_time": "18:25"
    },
    {
      "stop_name": "時任大附属中学校前 ",
      "arrived_time": "18:26"
    }
  ]
}
```

## LICENSE
Copyright (c) 2016 Ryo Sugimoto  
See LICENSE
