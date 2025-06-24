module github.com/bytedance/Elkeid/plugins/collector

go 1.23.0

replace github.com/bytedance/plugins => ../lib/go

require (
	github.com/GehirnInc/crypt v0.0.0-20200316065508-bb7000b8a962
	github.com/bytedance/plugins v0.0.0-20220826022814-07b31790f447
	github.com/cespare/xxhash/v2 v2.3.0
	github.com/coocood/freecache v1.2.3
	github.com/deckarep/golang-set v1.8.0
	github.com/docker/docker v25.0.6+incompatible
	github.com/go-logr/zapr v1.2.2
	github.com/jellydator/ttlcache/v3 v3.0.0
	github.com/juju/ratelimit v1.0.2
	github.com/karrick/godirwalk v1.16.1
	github.com/mitchellh/mapstructure v1.5.0
	github.com/robfig/cron/v3 v3.0.1
	github.com/shirou/gopsutil/v3 v3.22.10
	github.com/tklauser/go-sysconf v0.3.10
	github.com/vishvananda/netlink v1.2.0-beta
	go.uber.org/zap v1.20.0
	golang.org/x/sys v0.33.0
	google.golang.org/grpc v1.72.1
	k8s.io/cri-api v0.25.4
)

require (
	github.com/Microsoft/go-winio v0.6.0 // indirect
	github.com/containerd/log v0.1.0 // indirect
	github.com/distribution/reference v0.6.0 // indirect
	github.com/docker/go-connections v0.4.0 // indirect
	github.com/docker/go-units v0.5.0 // indirect
	github.com/felixge/httpsnoop v1.0.4 // indirect
	github.com/go-logr/logr v1.4.2 // indirect
	github.com/go-logr/stdr v1.2.2 // indirect
	github.com/go-ole/go-ole v1.2.6 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/lufia/plan9stats v0.0.0-20211012122336-39d0f177ccd0 // indirect
	github.com/moby/term v0.0.0-20221128092401-c43b287e0e0f // indirect
	github.com/morikuni/aec v1.0.0 // indirect
	github.com/opencontainers/go-digest v1.0.0 // indirect
	github.com/opencontainers/image-spec v1.0.2 // indirect
	github.com/pkg/errors v0.9.1 // indirect
	github.com/power-devops/perfstat v0.0.0-20210106213030-5aafc221ea8c // indirect
	github.com/tklauser/numcpus v0.4.0 // indirect
	github.com/vishvananda/netns v0.0.0-20211101163701-50045581ed74 // indirect
	github.com/yusufpapurcu/wmi v1.2.2 // indirect
	go.opentelemetry.io/auto/sdk v1.1.0 // indirect
	go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp v0.61.0 // indirect
	go.opentelemetry.io/otel v1.36.0 // indirect
	go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp v1.36.0 // indirect
	go.opentelemetry.io/otel/metric v1.36.0 // indirect
	go.opentelemetry.io/otel/trace v1.36.0 // indirect
	go.uber.org/atomic v1.9.0 // indirect
	go.uber.org/multierr v1.7.0 // indirect
	golang.org/x/mod v0.17.0 // indirect
	golang.org/x/net v0.40.0 // indirect
	golang.org/x/sync v0.14.0 // indirect
	golang.org/x/text v0.25.0 // indirect
	golang.org/x/time v0.2.0 // indirect
	golang.org/x/tools v0.21.1-0.20240508182429-e35e4ccd0d2d // indirect
	google.golang.org/genproto v0.0.0-20230410155749-daa745c078e1 // indirect
	google.golang.org/protobuf v1.36.6 // indirect
	gopkg.in/natefinch/lumberjack.v2 v2.0.0 // indirect
	gopkg.in/yaml.v2 v2.4.0 // indirect
	gotest.tools/v3 v3.4.0 // indirect
)
