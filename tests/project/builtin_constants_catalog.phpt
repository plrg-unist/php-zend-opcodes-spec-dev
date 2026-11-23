--TEST--
Namespaced lookup of predefined CLI constants preserves values, types and case
--FILE--
<?php
namespace Catalog;
echo ABDAY_1, "/", CASE_LOWER, "/", DATE_ATOM, "/", E_WARNING, "/", FILE_APPEND, "\n";
echo GLOB_MARK, "/", HASH_HMAC, "/", IMAGETYPE_PNG, "/", JSON_ERROR_DEPTH, "/", LC_ALL, "\n";
echo M_PI, "/", (NAN !== NAN), "/", PHP_VERSION_ID, "/", RADIXCHAR, "\n";
echo STREAM_CRYPTO_METHOD_TLSv1_2_CLIENT, "/", T_FMT, "/", UPLOAD_ERR_NO_FILE,
     "/", YESEXPR, "/", (ZEND_DEBUG_BUILD === false), "\n";
echo PCRE_VERSION, "/", MT_RAND_MT19937, "/", (\gettype(INF) === "double"), "\n";
echo (int) \defined("PHP_VERSION_ID"), (int) \defined("php_version_id"),
     (int) \defined("A!B"), (int) \defined("a_name_longer_than_any_predefined_constant"), "\n";
--EXPECT--
131072/0/Y-m-d\TH:i:sP/2/8
8/1/3/1/6
3.1415926535898/1/80600/65536
33/131114/4/327680/1
10.44 2024-06-07/0/1
1000
