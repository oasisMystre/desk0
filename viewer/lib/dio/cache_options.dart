import 'package:path_provider/path_provider.dart' show getTemporaryDirectory;
import 'package:http_cache_hive_store/http_cache_hive_store.dart'
    show HiveCacheStore;
import 'package:dio_cache_interceptor/dio_cache_interceptor.dart'
    show CacheOptions, CachePolicy;

Future<CacheOptions> createCacheOptions() async {
  final directory = await getTemporaryDirectory();

  return CacheOptions(
    policy: CachePolicy.request,
    hitCacheOnNetworkFailure: true,
    store: HiveCacheStore(directory.path),
  );
}

final cacheOptions = createCacheOptions();
