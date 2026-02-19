window.BENCHMARK_DATA = {
  "lastUpdate": 1771510677676,
  "repoUrl": "https://github.com/KingingWang/battery-estimator",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "kingingwang@foxmail.com",
            "name": "kingingwang",
            "username": "KingingWang"
          },
          "committer": {
            "email": "kingingwang@foxmail.com",
            "name": "kingingwang",
            "username": "KingingWang"
          },
          "distinct": true,
          "id": "f087fa2b3992d3e5ff9bccac8f02d74f6d9f15b5",
          "message": "refactor: remove unnecessary implementation comments",
          "timestamp": "2026-02-19T08:50:48+08:00",
          "tree_id": "3d351d63d35e5e8d460ade63260f652efb76a393",
          "url": "https://github.com/KingingWang/battery-estimator/commit/f087fa2b3992d3e5ff9bccac8f02d74f6d9f15b5"
        },
        "date": 1771462661247,
        "tool": "cargo",
        "benches": [
          {
            "name": "estimate_soc/lipo_3_7v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc/lifepo4_3_2v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc/liion_3_7v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc/conservative_3_77v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_fixed",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_with_temp/cold_temp",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_with_temp/normal_temp",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_with_temp/hot_temp",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_compensated",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve/voltage_to_soc",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve/voltage_to_soc_fixed",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve/voltage_range",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_new_small",
            "value": 24,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "curve_new_large",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/default_f32",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/default_fixed",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/custom_f32",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/custom_fixed",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "aging_compensation/f32",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "aging_compensation/fixed",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/new",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/from_fixed",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/voltage",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/soc",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/new_lipo",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/new_lifepo4",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/with_temperature_compensation",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/with_aging_compensation",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/with_all_compensation",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "custom_curve_estimate",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/min_voltage",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/max_voltage",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/below_min",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/above_max",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "throughput_100_estimations",
            "value": 2425,
            "range": "± 6",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kingingwang@foxmail.com",
            "name": "kingingwang",
            "username": "KingingWang"
          },
          "committer": {
            "email": "kingingwang@foxmail.com",
            "name": "kingingwang",
            "username": "KingingWang"
          },
          "distinct": true,
          "id": "6779d014e813a35c69f403abcf5c7e5a3fa3ac38",
          "message": "docs: cleanup unnecessary comments and add missing documentation\n\n- Remove implementation comments from src/curve.rs (convert to millivolts, cache values, binary search, etc.)\n- Remove implementation comments from src/estimator.rs (const function, clamp, apply compensation)\n- Add documentation for estimate_soc_compensated_fixed method in src/estimator.rs",
          "timestamp": "2026-02-19T22:11:03+08:00",
          "tree_id": "a0a33f594238d2a37d6f8aaf43dafe5fd48a7ee8",
          "url": "https://github.com/KingingWang/battery-estimator/commit/6779d014e813a35c69f403abcf5c7e5a3fa3ac38"
        },
        "date": 1771510677311,
        "tool": "cargo",
        "benches": [
          {
            "name": "estimate_soc/lipo_3_7v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc/lifepo4_3_2v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc/liion_3_7v",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc/conservative_3_77v",
            "value": 23,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_fixed",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_with_temp/cold_temp",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_with_temp/normal_temp",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_with_temp/hot_temp",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimate_soc_compensated",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve/voltage_to_soc",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve/voltage_to_soc_fixed",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve/voltage_range",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_new_small",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "curve_new_large",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/default_f32",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/default_fixed",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/custom_f32",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "temperature_compensation/custom_fixed",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "aging_compensation/f32",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "aging_compensation/fixed",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/new",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/from_fixed",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/voltage",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "curve_point/soc",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/new_lipo",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/new_lifepo4",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/with_temperature_compensation",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/with_aging_compensation",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "estimator_creation/with_all_compensation",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "custom_curve_estimate",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/min_voltage",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/max_voltage",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/below_min",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "boundary_cases/above_max",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "throughput_100_estimations",
            "value": 2423,
            "range": "± 20",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}