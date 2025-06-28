window.BENCHMARK_DATA = {
  "lastUpdate": 1751124625257,
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
      }
    ]
  }
}