window.BENCHMARK_DATA = {
  "lastUpdate": 1752912354427,
  "repoUrl": "https://github.com/mox692/tokio-monitor",
  "entries": {
    "Rust Benchmark": [
      {
        "commit": {
          "author": {
            "name": "mox692",
            "username": "mox692"
          },
          "committer": {
            "name": "mox692",
            "username": "mox692"
          },
          "id": "4d9749813c070fde3ae18bf896bba626a0beba5e",
          "message": "add bench",
          "timestamp": "2025-06-26T09:11:55Z",
          "url": "https://github.com/mox692/tokio-monitor/pull/29/commits/4d9749813c070fde3ae18bf896bba626a0beba5e"
        },
        "date": 1751124622926,
        "tool": "cargo",
        "benches": [
          {
            "name": "flight_record/spawn_1000_tasks - enable: true, flush: true",
            "value": 1988839,
            "range": "± 13490",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: true, flush: false",
            "value": 647204,
            "range": "± 67525",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: false, flush: true",
            "value": 2075058,
            "range": "± 62399",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: false, flush: false",
            "value": 697186,
            "range": "± 69140",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "mox692",
            "username": "mox692"
          },
          "committer": {
            "name": "mox692",
            "username": "mox692"
          },
          "id": "e773e9deb701b87f5e5de170eb10ed43de7becd4",
          "message": "add bench",
          "timestamp": "2025-06-26T09:11:55Z",
          "url": "https://github.com/mox692/tokio-monitor/pull/29/commits/e773e9deb701b87f5e5de170eb10ed43de7becd4"
        },
        "date": 1751125067541,
        "tool": "cargo",
        "benches": [
          {
            "name": "flight_record/spawn_1000_tasks - enable: true, flush: true",
            "value": 2421008,
            "range": "± 140036",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: true, flush: false",
            "value": 677475,
            "range": "± 59081",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: false, flush: true",
            "value": 2491018,
            "range": "± 28424",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: false, flush: false",
            "value": 700964,
            "range": "± 69452",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "mox692",
            "username": "mox692"
          },
          "committer": {
            "name": "mox692",
            "username": "mox692"
          },
          "id": "368ef3cbb6aa347624a10c45dd77a275cf7604da",
          "message": "add bench",
          "timestamp": "2025-06-26T09:11:55Z",
          "url": "https://github.com/mox692/tokio-monitor/pull/29/commits/368ef3cbb6aa347624a10c45dd77a275cf7604da"
        },
        "date": 1751184869443,
        "tool": "cargo",
        "benches": [
          {
            "name": "flight_record/spawn_1000_tasks - enable: true, flush: true",
            "value": 2078007,
            "range": "± 112840",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: true, flush: false",
            "value": 661772,
            "range": "± 47418",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: false, flush: true",
            "value": 2212247,
            "range": "± 69268",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - enable: false, flush: false",
            "value": 706403,
            "range": "± 58504",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "mox692",
            "username": "mox692"
          },
          "committer": {
            "name": "mox692",
            "username": "mox692"
          },
          "id": "a1eaca49ebd4765d720f016b526d8401ffd02111",
          "message": "add bench",
          "timestamp": "2025-07-05T10:53:15Z",
          "url": "https://github.com/mox692/tokio-monitor/pull/29/commits/a1eaca49ebd4765d720f016b526d8401ffd02111"
        },
        "date": 1751715056127,
        "tool": "cargo",
        "benches": [
          {
            "name": "flight_record/spawn_1000_tasks - trace: true, flush: true",
            "value": 2060282,
            "range": "± 82479",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - trace: true, flush: false",
            "value": 675906,
            "range": "± 57994",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - trace: false, flush: true",
            "value": 2185012,
            "range": "± 48983",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - trace: false, flush: false",
            "value": 722066,
            "range": "± 57089",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "moymoymox@gmail.com",
            "name": "Motoyuki Kimura",
            "username": "mox692"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7765d720d1a9ffea66af33ea0e69c559d9d52c4b",
          "message": "add bench (#29)",
          "timestamp": "2025-07-19T16:49:32+09:00",
          "tree_id": "aa47c0906231b6500b45e01605fa06e94cefe597",
          "url": "https://github.com/mox692/tokio-monitor/commit/7765d720d1a9ffea66af33ea0e69c559d9d52c4b"
        },
        "date": 1752912352398,
        "tool": "cargo",
        "benches": [
          {
            "name": "flight_record/spawn_1000_tasks - trace: true, flush: true",
            "value": 2075172,
            "range": "± 133351",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - trace: true, flush: false",
            "value": 682785,
            "range": "± 43918",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - trace: false, flush: true",
            "value": 2187867,
            "range": "± 78110",
            "unit": "ns/iter"
          },
          {
            "name": "flight_record/spawn_1000_tasks - trace: false, flush: false",
            "value": 721351,
            "range": "± 46352",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}