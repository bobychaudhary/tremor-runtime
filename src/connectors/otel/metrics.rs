// Copyright 2020-2021, The Tremor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{
    super::pb,
    common, id,
    resource::{self, resource_to_pb},
};
use crate::{connectors::pb::maybe_from_value, errors::Result};
use tremor_otelapis::opentelemetry::proto::{
    collector::metrics::v1::ExportMetricsServiceRequest,
    metrics::v1::{
        exemplar,
        metric::{self, Data},
        number_data_point,
        summary_data_point::ValueAtQuantile,
        Exemplar, Gauge, Histogram, HistogramDataPoint, InstrumentationLibraryMetrics,
        IntDataPoint, IntExemplar, IntGauge, IntHistogram, IntHistogramDataPoint, IntSum, Metric,
        NumberDataPoint, ResourceMetrics, Sum, Summary, SummaryDataPoint,
    },
};
use tremor_value::{literal, prelude::*, Value};

pub(crate) fn int_exemplars_to_json(data: Vec<IntExemplar>) -> Value<'static> {
    data.into_iter()
        .map(|exemplar| {
            literal!({
                "span_id": id::hex_span_id_to_json(&exemplar.span_id),
                "trace_id": id::hex_trace_id_to_json(&exemplar.trace_id),
                "filtered_labels": common::string_key_value_to_json(exemplar.filtered_labels),
                "time_unix_nano": exemplar.time_unix_nano,
                "value": exemplar.value
            })
        })
        .collect()
}

pub(crate) fn int_exemplars_to_pb(json: Option<&Value<'_>>) -> Result<Vec<IntExemplar>> {
    json.as_array()
        .ok_or("Unable to map json value to Exemplars pb")?
        .iter()
        .map(|data| {
            Ok(IntExemplar {
                filtered_labels: common::string_key_value_to_pb(data.get("filtered_labels"))?,
                time_unix_nano: pb::maybe_int_to_pbu64(data.get("time_unix_nano"))?,
                value: pb::maybe_int_to_pbi64(data.get("value"))?,
                span_id: id::hex_span_id_to_pb(data.get("span_id"))?,
                trace_id: id::hex_trace_id_to_pb(data.get("trace_id"))?,
            })
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_exemplars_to_json(data: Vec<Exemplar>) -> Value<'static> {
    data.into_iter()
        .map(|exemplar| {
            let mut filtered_attributes =
                common::key_value_list_to_json(exemplar.filtered_attributes);
            let mut filtered_labels = common::string_key_value_to_json(exemplar.filtered_labels);
            if let Some((attributes, labels)) = filtered_attributes
                .as_object_mut()
                .zip(filtered_labels.as_object_mut())
            {
                for (k, v) in labels.drain() {
                    attributes.insert(k, v);
                }
            };

            let mut r = literal!({
                "span_id": id::hex_span_id_to_json(&exemplar.span_id),
                "trace_id": id::hex_trace_id_to_json(&exemplar.trace_id),
                "filtered_attributes": filtered_attributes,
                "time_unix_nano": exemplar.time_unix_nano,
            });
            match exemplar.value {
                Some(exemplar::Value::AsDouble(v)) => {
                    r.try_insert("value", v);
                }
                Some(exemplar::Value::AsInt(v)) => {
                    r.try_insert("value", v);
                }
                None => (),
            };
            r
        })
        .collect()
}
#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_exemplars_to_pb(json: Option<&Value<'_>>) -> Result<Vec<Exemplar>> {
    json.as_array()
        .ok_or("Unable to map json value to Exemplars pb")?
        .iter()
        .map(|data| {
            let filtered_attributes = match (
                data.get_object("filtered_attributes"),
                data.get_object("filtered_labels"),
            ) {
                (None, None) => return Err("missing field `filtered_attributes`".into()),
                (Some(a), None) | (None, Some(a)) => common::obj_key_value_list_to_pb(a),
                (Some(a), Some(l)) => {
                    let mut a = common::obj_key_value_list_to_pb(a);
                    a.append(&mut common::obj_key_value_list_to_pb(l));
                    a
                }
            };
            Ok(Exemplar {
                filtered_attributes,
                filtered_labels: vec![],
                span_id: id::hex_span_id_to_pb(data.get("span_id"))?,
                trace_id: id::hex_trace_id_to_pb(data.get("trace_id"))?,
                time_unix_nano: pb::maybe_int_to_pbu64(data.get("time_unix_nano"))?,
                value: maybe_from_value(data.get("value"))?,
            })
        })
        .collect()
}

pub(crate) fn quantile_values_to_json(data: Vec<ValueAtQuantile>) -> Value<'static> {
    data.into_iter()
        .map(|data| {
            literal!({
                "value": data.value,
                "quantile": data.quantile,
            })
        })
        .collect()
}

pub(crate) fn quantile_values_to_pb(json: Option<&Value<'_>>) -> Result<Vec<ValueAtQuantile>> {
    json.as_array()
        .ok_or("Unable to map json value to ValueAtQuantiles")?
        .iter()
        .map(|data| {
            let value = pb::maybe_double_to_pb(data.get("value"))?;
            let quantile = pb::maybe_double_to_pb(data.get("quantile"))?;
            Ok(ValueAtQuantile { quantile, value })
        })
        .collect()
}

pub(crate) fn int_data_points_to_json(pb: Vec<IntDataPoint>) -> Value<'static> {
    pb.into_iter()
        .map(|data| {
            literal!({
                "value": data.value,
                "start_time_unix_nano": data.start_time_unix_nano,
                "time_unix_nano": data.time_unix_nano,
                "labels": common::string_key_value_to_json(data.labels),
                "exemplars": int_exemplars_to_json(data.exemplars),
            })
        })
        .collect()
}

pub(crate) fn int_data_points_to_pb(json: Option<&Value<'_>>) -> Result<Vec<IntDataPoint>> {
    json.as_array()
        .ok_or("Unable to map json value to otel pb IntDataPoint list")?
        .iter()
        .map(|item| {
            Ok(IntDataPoint {
                labels: common::string_key_value_to_pb(item.get("labels"))?,
                exemplars: int_exemplars_to_pb(item.get("exemplars"))?,
                time_unix_nano: pb::maybe_int_to_pbu64(item.get("time_unix_nano"))?,
                start_time_unix_nano: pb::maybe_int_to_pbu64(item.get("start_time_unix_nano"))?,
                value: pb::maybe_int_to_pbi64(item.get("value"))?,
            })
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_data_points_to_json(pb: Vec<NumberDataPoint>) -> Value<'static> {
    pb.into_iter()
        .map(|data| {
            let mut attributes = common::key_value_list_to_json(data.attributes);
            let mut labels = common::string_key_value_to_json(data.labels);
            if let Some((attributes, labels)) =
                attributes.as_object_mut().zip(labels.as_object_mut())
            {
                for (k, v) in labels.drain() {
                    attributes.insert(k, v);
                }
            };
            let mut r = literal!({
                "start_time_unix_nano": data.start_time_unix_nano,
                "time_unix_nano": data.time_unix_nano,
                "attributes": attributes,
                "exemplars": double_exemplars_to_json(data.exemplars),
            });
            match data.value {
                Some(number_data_point::Value::AsDouble(v)) => {
                    r.try_insert("value", v);
                }
                Some(number_data_point::Value::AsInt(v)) => {
                    r.try_insert("value", v);
                }
                None => (),
            };
            r
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_data_points_to_pb(json: Option<&Value<'_>>) -> Result<Vec<NumberDataPoint>> {
    json.as_array()
        .ok_or("Unable to map json value to otel pb NumberDataPoint list")?
        .iter()
        .map(|data| {
            let attributes = common::get_attributes_or_labes(data)?;
            Ok(NumberDataPoint {
                labels: vec![],
                attributes,
                exemplars: double_exemplars_to_pb(data.get("exemplars"))?,
                time_unix_nano: pb::maybe_int_to_pbu64(data.get("time_unix_nano"))?,
                start_time_unix_nano: pb::maybe_int_to_pbu64(data.get("start_time_unix_nano"))?,
                value: maybe_from_value(data.get("value"))?,
            })
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_histo_data_points_to_json(pb: Vec<HistogramDataPoint>) -> Value<'static> {
    pb.into_iter()
        .map(|point| {
            let mut attributes = common::key_value_list_to_json(point.attributes);
            let mut labels = common::string_key_value_to_json(point.labels);
            if let Some((attributes, labels)) =
                attributes.as_object_mut().zip(labels.as_object_mut())
            {
                for (k, v) in labels.drain() {
                    attributes.insert(k, v);
                }
            };
            literal!({
                "start_time_unix_nano": point.start_time_unix_nano,
                "time_unix_nano": point.time_unix_nano,
                "attributes": attributes,
                "exemplars": double_exemplars_to_json(point.exemplars),
                "sum": point.sum,
                "count": point.count,
                "explicit_bounds": point.explicit_bounds,
                "bucket_counts": point.bucket_counts,
            })
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_histo_data_points_to_pb(
    json: Option<&Value<'_>>,
) -> Result<Vec<HistogramDataPoint>> {
    json.as_array()
        .ok_or("Unable to map json value to otel pb HistogramDataPoint list")?
        .iter()
        .map(|data| {
            let attributes = common::get_attributes_or_labes(data)?;
            Ok(HistogramDataPoint {
                labels: vec![],
                attributes,
                time_unix_nano: pb::maybe_int_to_pbu64(data.get("time_unix_nano"))?,
                start_time_unix_nano: pb::maybe_int_to_pbu64(data.get("start_time_unix_nano"))?,
                sum: pb::maybe_double_to_pb(data.get("sum"))?,
                count: pb::maybe_int_to_pbu64(data.get("count"))?,
                exemplars: double_exemplars_to_pb(data.get("exemplars"))?,
                explicit_bounds: pb::f64_repeated_to_pb(data.get("explicit_bounds"))?,
                bucket_counts: pb::u64_repeated_to_pb(data.get("explicit_bounds"))?,
            })
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_summary_data_points_to_json(pb: Vec<SummaryDataPoint>) -> Value<'static> {
    pb.into_iter()
        .map(|point| {
            let mut attributes = common::key_value_list_to_json(point.attributes);
            let mut labels = common::string_key_value_to_json(point.labels);
            if let Some((attributes, labels)) =
                attributes.as_object_mut().zip(labels.as_object_mut())
            {
                for (k, v) in labels.drain() {
                    attributes.insert(k, v);
                }
            };
            literal!({
                "start_time_unix_nano": point.start_time_unix_nano,
                "time_unix_nano": point.time_unix_nano,
                "attributes": attributes,
                "quantile_values": quantile_values_to_json(point.quantile_values),
                "sum": point.sum,
                "count": point.count,
            })
        })
        .collect()
}

#[allow(deprecated)] // handling depricated fields is required by the PB files
pub(crate) fn double_summary_data_points_to_pb(
    json: Option<&Value<'_>>,
) -> Result<Vec<SummaryDataPoint>> {
    json.as_array()
        .ok_or("Unable to map json value to otel pb SummaryDataPoint list")?
        .iter()
        .map(|data| {
            let attributes = common::get_attributes_or_labes(data)?;
            Ok(SummaryDataPoint {
                labels: vec![],
                attributes,
                time_unix_nano: pb::maybe_int_to_pbu64(data.get("time_unix_nano"))?,
                start_time_unix_nano: pb::maybe_int_to_pbu64(data.get("start_time_unix_nano"))?,
                sum: pb::maybe_double_to_pb(data.get("sum"))?,
                count: pb::maybe_int_to_pbu64(data.get("count"))?,
                quantile_values: quantile_values_to_pb(data.get("quantile_values"))?,
            })
        })
        .collect()
}

pub(crate) fn int_histo_data_points_to_json(pb: Vec<IntHistogramDataPoint>) -> Value<'static> {
    pb.into_iter()
        .map(|points| {
            literal!({
                "start_time_unix_nano": points.start_time_unix_nano,
                "time_unix_nano": points.time_unix_nano,
                "labels": common::string_key_value_to_json(points.labels),
                "exemplars": int_exemplars_to_json(points.exemplars),
                "sum": points.sum,
                "count": points.count,
                "explicit_bounds": points.explicit_bounds,
                "bucket_counts": points.bucket_counts,
            })
        })
        .collect()
}

pub(crate) fn int_histo_data_points_to_pb(
    json: Option<&Value<'_>>,
) -> Result<Vec<IntHistogramDataPoint>> {
    json.as_array()
        .ok_or("Unable to map json value to otel pb IntHistogramDataPoint list")?
        .iter()
        .map(|item| {
            Ok(IntHistogramDataPoint {
                labels: common::string_key_value_to_pb(item.get("labels"))?,
                time_unix_nano: pb::maybe_int_to_pbu64(item.get("time_unix_nano"))?,
                start_time_unix_nano: pb::maybe_int_to_pbu64(item.get("start_time_unix_nano"))?,
                sum: pb::maybe_int_to_pbi64(item.get("sum"))?,
                count: pb::maybe_int_to_pbu64(item.get("count"))?,
                exemplars: int_exemplars_to_pb(item.get("exemplars"))?,
                explicit_bounds: pb::f64_repeated_to_pb(item.get("explicit_bounds"))?,
                bucket_counts: pb::u64_repeated_to_pb(item.get("explicit_bounds"))?,
            })
        })
        .collect()
}

pub(crate) fn int_sum_data_points_to_json(pb: Vec<IntDataPoint>) -> Value<'static> {
    int_data_points_to_json(pb)
}

pub(crate) fn metrics_data_to_json(pb: Option<metric::Data>) -> Value<'static> {
    pb.map(|pb| match pb {
        Data::IntGauge(data) => literal!({
            "int-gauge": {
            "data_points":  int_data_points_to_json(data.data_points)
        }}),
        Data::Sum(data) => literal!({
            "double-sum": {
            "is_monotonic": data.is_monotonic,
            "data_points":  double_data_points_to_json(data.data_points),
            "aggregation_temporality": data.aggregation_temporality,
        }}),
        Data::Gauge(data) => literal!({
            "double-gauge": {
            "data_points":  double_data_points_to_json(data.data_points),
        }}),
        Data::Histogram(data) => literal!({
            "double-histogram": {
            "data_points":  double_histo_data_points_to_json(data.data_points),
            "aggregation_temporality": data.aggregation_temporality,
        }}),
        Data::Summary(data) => literal!({
            "double-summary": {
            "data_points":  double_summary_data_points_to_json(data.data_points),
        }}),
        Data::IntHistogram(data) => literal!({
            "int-histogram": {
            "data_points":  int_histo_data_points_to_json(data.data_points),
            "aggregation_temporality": data.aggregation_temporality,
        }}),
        Data::IntSum(data) => literal!({
            "int-sum": {
            "is_monotonic": data.is_monotonic,
            "data_points":  int_sum_data_points_to_json(data.data_points),
            "aggregation_temporality": data.aggregation_temporality,
            }
        }),
    })
    .unwrap_or_default()
}

pub(crate) fn metrics_data_to_pb(data: &Value<'_>) -> Result<metric::Data> {
    if let Some(json) = data.get_object("int-gauge") {
        let data_points = int_data_points_to_pb(json.get("data_points"))?;
        Ok(metric::Data::IntGauge(IntGauge { data_points }))
    } else if let Some(json) = data.get_object("double-gauge") {
        let data_points = double_data_points_to_pb(json.get("data_points"))?;
        Ok(metric::Data::Gauge(Gauge { data_points }))
    } else if let Some(json) = data.get_object("int-sum") {
        let data_points = int_data_points_to_pb(json.get("data_points"))?;
        let is_monotonic = pb::maybe_bool_to_pb(json.get("is_monotonic"))?;
        let aggregation_temporality = pb::maybe_int_to_pbi32(json.get("aggregation_temporality"))?;
        Ok(metric::Data::IntSum(IntSum {
            data_points,
            aggregation_temporality,
            is_monotonic,
        }))
    } else if let Some(json) = data.get_object("double-sum") {
        let data_points = double_data_points_to_pb(json.get("data_points"))?;
        let is_monotonic = pb::maybe_bool_to_pb(json.get("is_monotonic"))?;
        let aggregation_temporality = pb::maybe_int_to_pbi32(json.get("aggregation_temporality"))?;
        Ok(metric::Data::Sum(Sum {
            data_points,
            aggregation_temporality,
            is_monotonic,
        }))
    } else if let Some(json) = data.get_object("int-histogram") {
        let data_points = int_histo_data_points_to_pb(json.get("data_points"))?;
        let aggregation_temporality = pb::maybe_int_to_pbi32(json.get("aggregation_temporality"))?;
        Ok(metric::Data::IntHistogram(IntHistogram {
            data_points,
            aggregation_temporality,
        }))
    } else if let Some(json) = data.get_object("double-histogram") {
        let data_points = double_histo_data_points_to_pb(json.get("data_points"))?;
        let aggregation_temporality = pb::maybe_int_to_pbi32(json.get("aggregation_temporality"))?;
        Ok(metric::Data::Histogram(Histogram {
            data_points,
            aggregation_temporality,
        }))
    } else if let Some(json) = data.get_object("double-summary") {
        let data_points = double_summary_data_points_to_pb(json.get("data_points"))?;
        Ok(metric::Data::Summary(Summary { data_points }))
    } else {
        Err("Invalid metric data point type - cannot convert to pb".into())
    }
}

fn metric_to_json(metric: Metric) -> Value<'static> {
    literal!({
        "name": metric.name,
        "description": metric.description,
        "data": metrics_data_to_json(metric.data),
        "unit": metric.unit,
    })
}

pub(crate) fn instrumentation_library_metrics_to_json<'event>(
    pb: Vec<tremor_otelapis::opentelemetry::proto::metrics::v1::InstrumentationLibraryMetrics>,
) -> Value<'event> {
    let mut json = Vec::with_capacity(pb.len());
    for data in pb {
        let metrics: Value = data.metrics.into_iter().map(metric_to_json).collect();
        let mut e = literal!({ "metrics": metrics, "schema_url": data.schema_url });
        if let Some(il) = data.instrumentation_library {
            let il = common::maybe_instrumentation_library_to_json(il);
            e.try_insert("instrumentation_library", il);
        }
        json.push(e);
    }

    literal!(json)
}

pub(crate) fn instrumentation_library_metrics_to_pb(
    data: Option<&Value<'_>>,
) -> Result<Vec<InstrumentationLibraryMetrics>> {
    let data = data
        .as_array()
        .ok_or("Invalid json mapping for InstrumentationLibraryMetrics")?;
    let mut pb = Vec::with_capacity(data.len());
    for data in data.iter() {
        let mut metrics = Vec::new();
        if let Some(data) = data.get_array("metrics") {
            for metric in data {
                metrics.push(Metric {
                    name: pb::maybe_string_to_pb(metric.get("name"))?,
                    description: pb::maybe_string_to_pb(metric.get("description"))?,
                    unit: pb::maybe_string_to_pb(metric.get("unit"))?,
                    data: metric.get("data").map(metrics_data_to_pb).transpose()?,
                });
            }
        }

        let e = InstrumentationLibraryMetrics {
            schema_url: data
                .get_str("schema_url")
                .map(ToString::to_string)
                .unwrap_or_default(),
            instrumentation_library: data
                .get("instrumentation_library")
                .map(common::instrumentation_library_to_pb)
                .transpose()?,
            metrics,
        };
        pb.push(e);
    }
    Ok(pb)
}

pub(crate) fn resource_metrics_to_json(request: ExportMetricsServiceRequest) -> Value<'static> {
    let metrics: Value = request
        .resource_metrics
        .into_iter()
        .map(|metric| {
            let ill =
                instrumentation_library_metrics_to_json(metric.instrumentation_library_metrics);
            let mut base = literal!({ "instrumentation_library_metrics": ill,  "schema_url": metric.schema_url });
            if let Some(r) = metric.resource {
                base.try_insert("resource", resource::resource_to_json(r));
            };
            base
        })
        .collect();

    literal!({ "metrics": metrics })
}

pub(crate) fn resource_metrics_to_pb(json: Option<&Value<'_>>) -> Result<Vec<ResourceMetrics>> {
    json.get_array("metrics")
        .ok_or("Invalid json mapping for otel metrics message - cannot convert to pb")?
        .iter()
        .filter_map(Value::as_object)
        .map(|item| {
            Ok(ResourceMetrics {
                schema_url: item
                    .get("schema_url")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                instrumentation_library_metrics: instrumentation_library_metrics_to_pb(
                    item.get("instrumentation_library_metrics"),
                )?,
                resource: item.get("resource").map(resource_to_pb).transpose()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)] // This is just for tests
    use tremor_otelapis::opentelemetry::proto::{
        common::v1::InstrumentationLibrary, resource::v1::Resource,
    };
    use tremor_script::utils::sorted_serialize;

    use super::*;

    #[test]
    fn int_exemplars() -> Result<()> {
        let nanos = tremor_common::time::nanotime();
        let span_id_pb = id::random_span_id_bytes(nanos);
        let span_id_json = id::test::pb_span_id_to_json(&span_id_pb);
        let trace_id_json = id::random_trace_id_value(nanos);
        let trace_id_pb = id::test::json_trace_id_to_pb(Some(&trace_id_json))?;

        let pb = vec![IntExemplar {
            span_id: span_id_pb.clone(),
            trace_id: trace_id_pb,
            time_unix_nano: 0,
            filtered_labels: vec![],
            value: 42,
        }];
        let json = int_exemplars_to_json(pb.clone());
        let back_again = int_exemplars_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "time_unix_nano": 0,
            "span_id": span_id_json,
            "trace_id": trace_id_json,
            "filtered_labels": {},
            "value": 42
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = int_exemplars_to_json(vec![]);
        let back_again = int_exemplars_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn double_exemplars() -> Result<()> {
        let nanos = tremor_common::time::nanotime();
        let span_id_pb = id::random_span_id_bytes(nanos);
        let span_id_json = id::test::pb_span_id_to_json(&span_id_pb);
        let trace_id_json = id::random_trace_id_value(nanos);
        let trace_id_pb = id::test::json_trace_id_to_pb(Some(&trace_id_json))?;

        let pb = vec![Exemplar {
            filtered_attributes: vec![],
            span_id: span_id_pb.clone(),
            trace_id: trace_id_pb,
            time_unix_nano: 0,
            filtered_labels: vec![],
            value: maybe_from_value(Some(&Value::from(42.42)))?,
        }];
        let json = double_exemplars_to_json(pb.clone());
        let back_again = double_exemplars_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "time_unix_nano": 0,
            "span_id": span_id_json,
            "trace_id": trace_id_json,
            "filtered_attributes": {},
            "value": 42.42
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = double_exemplars_to_json(vec![]);
        let back_again = double_exemplars_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn quantile_values() -> Result<()> {
        let pb = vec![ValueAtQuantile {
            value: 42.42,
            quantile: 0.3,
        }];
        let json = quantile_values_to_json(pb.clone());
        let back_again = quantile_values_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "value": 42.42,
            "quantile": 0.3,
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = quantile_values_to_json(vec![]);
        let back_again = quantile_values_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn int_data_points() -> Result<()> {
        let pb = vec![IntDataPoint {
            value: 42,
            start_time_unix_nano: 0,
            time_unix_nano: 0,
            labels: vec![],
            exemplars: vec![],
        }];
        let json = int_data_points_to_json(pb.clone());
        let back_again = int_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "value": 42,
            "start_time_unix_nano": 0,
            "time_unix_nano": 0,
            "labels": {},
            "exemplars": []
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = int_data_points_to_json(vec![]);
        let back_again = int_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn double_data_points() -> Result<()> {
        let pb = vec![NumberDataPoint {
            attributes: vec![],
            value: maybe_from_value(Some(&Value::from(42.42)))?,
            start_time_unix_nano: 0,
            time_unix_nano: 0,
            labels: vec![],
            exemplars: vec![],
        }];
        let json = double_data_points_to_json(pb.clone());
        let back_again = double_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "value": 42.42,
            "start_time_unix_nano": 0,
            "time_unix_nano": 0,
            "attributes": {},
            "exemplars": []
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = double_data_points_to_json(vec![]);
        let back_again = double_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn int_histo_data_points() -> Result<()> {
        let pb = vec![IntHistogramDataPoint {
            start_time_unix_nano: 0,
            time_unix_nano: 0,
            labels: vec![],
            exemplars: vec![],
            sum: 0,
            count: 0,
            explicit_bounds: vec![],
            bucket_counts: vec![],
        }];
        let json = int_histo_data_points_to_json(pb.clone());
        let back_again = int_histo_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "start_time_unix_nano": 0,
            "time_unix_nano": 0,
            "labels": {},
            "exemplars": [],
            "sum": 0,
            "count": 0,
            "explicit_bounds": [],
            "bucket_counts": [],
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = int_histo_data_points_to_json(vec![]);
        let back_again = int_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn double_histo_data_points() -> Result<()> {
        let pb = vec![HistogramDataPoint {
            attributes: vec![],
            start_time_unix_nano: 0,
            time_unix_nano: 0,
            labels: vec![],
            exemplars: vec![],
            sum: 0.0,
            count: 0,
            explicit_bounds: vec![],
            bucket_counts: vec![],
        }];
        let json = double_histo_data_points_to_json(pb.clone());
        let back_again = double_histo_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "start_time_unix_nano": 0,
            "time_unix_nano": 0,
            "attributes": {},
            "exemplars": [],
            "sum": 0.0,
            "count": 0,
            "explicit_bounds": [],
            "bucket_counts": [],
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = double_histo_data_points_to_json(vec![]);
        let back_again = double_histo_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn double_summary_data_points() -> Result<()> {
        let pb = vec![SummaryDataPoint {
            attributes: vec![],
            start_time_unix_nano: 0,
            time_unix_nano: 0,
            labels: vec![],
            sum: 0.0,
            count: 0,
            quantile_values: vec![ValueAtQuantile {
                value: 0.1,
                quantile: 0.2,
            }],
        }];
        let json = double_summary_data_points_to_json(pb.clone());
        let back_again = double_summary_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "start_time_unix_nano": 0,
            "time_unix_nano": 0,
            "attributes": {},
            "sum": 0.0,
            "count": 0,
            "quantile_values": [ { "value": 0.1, "quantile": 0.2 }]
        }]);
        assert_eq!(expected, json);
        assert_eq!(pb, back_again);

        // Empty
        let json = double_summary_data_points_to_json(vec![]);
        let back_again = double_summary_data_points_to_pb(Some(&json))?;
        let expected: Value = literal!([]);
        assert_eq!(expected, json);
        assert_eq!(back_again, vec![]);

        Ok(())
    }

    #[test]
    fn metrics_data_int_gauge() -> Result<()> {
        let pb = Some(metric::Data::IntGauge(IntGauge {
            data_points: vec![IntDataPoint {
                value: 42,
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                exemplars: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "int-gauge": {
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "labels": {},
                    "exemplars": [],
                    "value": 42
                }]
        }});
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn metrics_data_double_sum() -> Result<()> {
        let pb = Some(metric::Data::Sum(Sum {
            is_monotonic: false,
            aggregation_temporality: 0,
            data_points: vec![NumberDataPoint {
                attributes: vec![],
                value: maybe_from_value(Some(&Value::from(43.43)))?,
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                exemplars: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "double-sum": {
                "is_monotonic": false,
                "aggregation_temporality": 0,
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "attributes": {},
                    "exemplars": [],
                    "value": 43.43
                }]
        }});
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn metrics_data_double_gauge() -> Result<()> {
        let pb = Some(metric::Data::Gauge(Gauge {
            data_points: vec![NumberDataPoint {
                attributes: vec![],
                value: maybe_from_value(Some(&Value::from(43.43)))?,
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                exemplars: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "double-gauge": {
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "attributes": {},
                    "exemplars": [],
                    "value": 43.43
                }]
        }});
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn metrics_data_double_histo() -> Result<()> {
        let pb = Some(metric::Data::Histogram(Histogram {
            aggregation_temporality: 0,
            data_points: vec![HistogramDataPoint {
                attributes: vec![],
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                exemplars: vec![],
                count: 5,
                sum: 10.0,
                bucket_counts: vec![],
                explicit_bounds: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "double-histogram": {
                "aggregation_temporality": 0,
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "attributes": {},
                    "exemplars": [],
                    "sum": 10.0,
                    "count": 5,
                    "bucket_counts": [],
                    "explicit_bounds": []
                }]
            }
        });
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn metrics_data_double_summary() -> Result<()> {
        let pb = Some(metric::Data::Summary(Summary {
            data_points: vec![SummaryDataPoint {
                attributes: vec![],
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                count: 0,
                sum: 0.0,
                quantile_values: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "double-summary": {
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "attributes": {},
                    "count": 0,
                    "sum": 0.0,
                    "quantile_values": []
                }]
        }});
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn metrics_data_int_histo() -> Result<()> {
        let pb = Some(metric::Data::IntHistogram(IntHistogram {
            aggregation_temporality: 0,
            data_points: vec![IntHistogramDataPoint {
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                exemplars: vec![],
                count: 5,
                sum: 10,
                bucket_counts: vec![],
                explicit_bounds: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "int-histogram": {
                "aggregation_temporality": 0,
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "labels": {},
                    "exemplars": [],
                    "count": 5,
                    "sum": 10,
                    "bucket_counts": [],
                    "explicit_bounds": []
                }]
        }});
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn metrics_data_int_sum() -> Result<()> {
        let pb = Some(metric::Data::IntSum(IntSum {
            is_monotonic: false,
            aggregation_temporality: 0,
            data_points: vec![IntDataPoint {
                value: 4,
                start_time_unix_nano: 0,
                time_unix_nano: 0,
                labels: vec![],
                exemplars: vec![],
            }],
        }));

        let json = metrics_data_to_json(pb.clone());
        let back_again = metrics_data_to_pb(&json)?;
        let expected: Value = literal!({
            "int-sum": {
                "is_monotonic": false,
                "aggregation_temporality": 0,
                "data_points": [{
                    "start_time_unix_nano": 0,
                    "time_unix_nano": 0,
                    "labels": {},
                    "exemplars": [],
                    "value": 4
                }]
        }});
        assert_eq!(expected, json);
        assert_eq!(pb, Some(back_again));
        Ok(())
    }

    #[test]
    fn instrumentation_library_metrics() -> Result<()> {
        let pb = vec![InstrumentationLibraryMetrics {
            schema_url: "schema_url".into(),
            instrumentation_library: Some(InstrumentationLibrary {
                name: "name".into(),
                version: "v0.1.2".into(),
            }),
            metrics: vec![Metric {
                name: "test".into(),
                description: "blah blah blah blah".into(),
                unit: "badgerfeet".into(),
                data: Some(metric::Data::IntGauge(IntGauge {
                    data_points: vec![IntDataPoint {
                        value: 42,
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        labels: vec![],
                        exemplars: vec![],
                    }],
                })),
            }],
        }];
        let json = instrumentation_library_metrics_to_json(pb.clone());
        let back_again = instrumentation_library_metrics_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "instrumentation_library": { "name": "name", "version": "v0.1.2" },
            "schema_url": "schema_url",
            "metrics": [{
                "name": "test",
                "description": "blah blah blah blah",
                "unit": "badgerfeet",
                "data": {
                    "int-gauge": {
                        "data_points": [{
                            "start_time_unix_nano": 0,
                            "time_unix_nano": 0,
                            "labels": {},
                            "exemplars": [],
                            "value": 42
                        }]
                    }
                },
            }]
        }]);

        assert_eq!(sorted_serialize(&expected)?, sorted_serialize(&json)?);
        assert_eq!(pb, back_again);

        Ok(())
    }

    #[test]
    fn instrumentation_library_metrics_nolib() -> Result<()> {
        let pb = vec![InstrumentationLibraryMetrics {
            schema_url: "schema_url".into(),
            instrumentation_library: None,
            metrics: vec![Metric {
                name: "test".into(),
                description: "blah blah blah blah".into(),
                unit: "badgerfeet".into(),
                data: Some(metric::Data::IntGauge(IntGauge {
                    data_points: vec![IntDataPoint {
                        value: 42,
                        start_time_unix_nano: 0,
                        time_unix_nano: 0,
                        labels: vec![],
                        exemplars: vec![],
                    }],
                })),
            }],
        }];
        let json = instrumentation_library_metrics_to_json(pb.clone());
        let back_again = instrumentation_library_metrics_to_pb(Some(&json))?;
        let expected: Value = literal!([{
            "schema_url": "schema_url",
            "metrics": [{
                "name": "test",
                "description": "blah blah blah blah",
                "unit": "badgerfeet",
                "data": {
                    "int-gauge": {
                        "data_points": [{
                            "start_time_unix_nano": 0,
                            "time_unix_nano": 0,
                            "labels": {},
                            "exemplars": [],
                            "value": 42
                        }]
                    }
                },
            }]
        }]);

        assert_eq!(sorted_serialize(&expected)?, sorted_serialize(&json)?);
        assert_eq!(pb, back_again);

        Ok(())
    }

    #[test]
    fn resource_metrics() -> Result<()> {
        let pb = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                schema_url: "schema_url".into(),
                resource: Some(Resource {
                    attributes: vec![],
                    dropped_attributes_count: 8,
                }),
                instrumentation_library_metrics: vec![InstrumentationLibraryMetrics {
                    schema_url: "schema_url".into(),
                    instrumentation_library: Some(InstrumentationLibrary {
                        name: "name".into(),
                        version: "v0.1.2".into(),
                    }), // TODO For now its an error for this to be None - may need to revisit
                    metrics: vec![Metric {
                        name: "test".into(),
                        description: "blah blah blah blah".into(),
                        unit: "badgerfeet".into(),
                        data: Some(metric::Data::IntGauge(IntGauge {
                            data_points: vec![IntDataPoint {
                                value: 42,
                                start_time_unix_nano: 0,
                                time_unix_nano: 0,
                                labels: vec![],
                                exemplars: vec![],
                            }],
                        })),
                    }],
                }],
            }],
        };
        let json = resource_metrics_to_json(pb.clone());
        let back_again = resource_metrics_to_pb(Some(&json))?;
        let expected: Value = literal!({
            "metrics": [
                {
                    "resource": { "attributes": {}, "dropped_attributes_count": 8 },
                    "schema_url": "schema_url",
                    "instrumentation_library_metrics": [{
                            "instrumentation_library": { "name": "name", "version": "v0.1.2" },
                            "schema_url": "schema_url",
                            "metrics": [{
                                "name": "test",
                                "description": "blah blah blah blah",
                                "unit": "badgerfeet",
                                "data": {
                                    "int-gauge": {
                                        "data_points": [{
                                            "start_time_unix_nano": 0,
                                            "time_unix_nano": 0,
                                            "labels": {},
                                            "exemplars": [],
                                            "value": 42
                                        }]
                                    }
                                },
                            }]
                    }]
                }
            ]
        });

        assert_eq!(sorted_serialize(&expected)?, sorted_serialize(&json)?);
        assert_eq!(pb.resource_metrics, back_again);

        Ok(())
    }
}
