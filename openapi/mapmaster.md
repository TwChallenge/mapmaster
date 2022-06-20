---
title: mapmaster v0.1.0
language_tabs:
  - shell: Shell
toc_footers: []
includes: []
search: true
highlight_theme: darkula
headingLevel: 2

---

<!-- Generator: Widdershins v4.0.1 -->

<h1 id="mapmaster">mapmaster v0.1.0</h1>

> Scroll down for code samples, example requests and responses. Select a language for code samples from the tabs above or the mobile navigation menu.

Our tool to manage teeworlds maps in a distributed manner.

Base URLs:

* <a href="/mapmaster">/mapmaster</a>

Web: <a href="https://github.com/TwChallenge/mapmaster">Homepage</a> 

# Authentication

* API Key (ApiKeyAuth)
    - Parameter Name: **x-api-key**, in: header. Requires an API key to access.

<h1 id="mapmaster-default">Default</h1>

## list_maps

<a id="opIdlist_maps"></a>

> Code samples

```shell
# You can also use wget
curl -X GET /mapmaster/list \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`GET /list`

<h3 id="list_maps-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|name|query|string|false|none|
|map_state|query|[MapState](#schemamapstate)|false|none|
|difficulty|query|[Difficulty](#schemadifficulty)|false|none|

#### Enumerated Values

|Parameter|Value|
|---|---|
|map_state|new|
|map_state|declined|
|map_state|approved|
|map_state|published|
|difficulty|easy|
|difficulty|main|
|difficulty|hard|
|difficulty|insane|

> Example responses

> 200 Response

```json
[
  {
    "name": "string",
    "difficulty": "easy",
    "state": "new",
    "created_at": 0,
    "last_changed": 0
  }
]
```

<h3 id="list_maps-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|Inline|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|

<h3 id="list_maps-responseschema">Response Schema</h3>

Status Code **200**

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|[[Map](#schemamap)]|false|none|none|
|» name|string|true|none|none|
|» difficulty|[Difficulty](#schemadifficulty)|true|none|none|
|» state|[MapState](#schemamapstate)|true|none|none|
|» created_at|integer(uint64)|true|none|none|
|» last_changed|integer(uint64)|true|none|none|

#### Enumerated Values

|Property|Value|
|---|---|
|difficulty|easy|
|difficulty|main|
|difficulty|hard|
|difficulty|insane|
|state|new|
|state|declined|
|state|approved|
|state|published|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

## create_map

<a id="opIdcreate_map"></a>

> Code samples

```shell
# You can also use wget
curl -X POST /mapmaster/create \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`POST /create`

> Body parameter

```json
{
  "name": "string",
  "difficulty": "string",
  "url": "string"
}
```

<h3 id="create_map-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[CreateMapData](#schemacreatemapdata)|true|none|

> Example responses

> 400 Response

```json
{
  "err": "string",
  "msg": "string"
}
```

<h3 id="create_map-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|None|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|
|default|Default|none|[CustomError](#schemacustomerror)|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

## change_map_difficulty

<a id="opIdchange_map_difficulty"></a>

> Code samples

```shell
# You can also use wget
curl -X POST /mapmaster/change_difficulty \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`POST /change_difficulty`

> Body parameter

```json
{
  "name": "string",
  "difficulty": "string"
}
```

<h3 id="change_map_difficulty-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[ChangeMapDifficultyData](#schemachangemapdifficultydata)|true|none|

> Example responses

> 400 Response

```json
{
  "err": "string",
  "msg": "string"
}
```

<h3 id="change_map_difficulty-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|None|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|
|default|Default|none|[CustomError](#schemacustomerror)|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

## approve_map

<a id="opIdapprove_map"></a>

> Code samples

```shell
# You can also use wget
curl -X POST /mapmaster/approve \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`POST /approve`

> Body parameter

```json
{
  "name": "string"
}
```

<h3 id="approve_map-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[JustTheMapName](#schemajustthemapname)|true|none|

> Example responses

> 400 Response

```json
{
  "err": "string",
  "msg": "string"
}
```

<h3 id="approve_map-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|None|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|
|default|Default|none|[CustomError](#schemacustomerror)|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

## publish_map

<a id="opIdpublish_map"></a>

> Code samples

```shell
# You can also use wget
curl -X POST /mapmaster/publish \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`POST /publish`

> Body parameter

```json
{
  "name": "string"
}
```

<h3 id="publish_map-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[JustTheMapName](#schemajustthemapname)|true|none|

> Example responses

> 400 Response

```json
{
  "err": "string",
  "msg": "string"
}
```

<h3 id="publish_map-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|None|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|
|default|Default|none|[CustomError](#schemacustomerror)|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

## recall_map

<a id="opIdrecall_map"></a>

> Code samples

```shell
# You can also use wget
curl -X POST /mapmaster/recall \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`POST /recall`

> Body parameter

```json
{
  "name": "string"
}
```

<h3 id="recall_map-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[JustTheMapName](#schemajustthemapname)|true|none|

> Example responses

> 400 Response

```json
{
  "err": "string",
  "msg": "string"
}
```

<h3 id="recall_map-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|None|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|
|default|Default|none|[CustomError](#schemacustomerror)|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

## decline_map

<a id="opIddecline_map"></a>

> Code samples

```shell
# You can also use wget
curl -X POST /mapmaster/decline \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'x-api-key: API_KEY'

```

`POST /decline`

> Body parameter

```json
{
  "name": "string"
}
```

<h3 id="decline_map-parameters">Parameters</h3>

|Name|In|Type|Required|Description|
|---|---|---|---|---|
|body|body|[JustTheMapName](#schemajustthemapname)|true|none|

> Example responses

> 400 Response

```json
{
  "err": "string",
  "msg": "string"
}
```

<h3 id="decline_map-responses">Responses</h3>

|Status|Meaning|Description|Schema|
|---|---|---|---|
|200|[OK](https://tools.ietf.org/html/rfc7231#section-6.3.1)|none|None|
|400|[Bad Request](https://tools.ietf.org/html/rfc7231#section-6.5.1)|# 400 Bad Request
The request given is wrongly formatted or data was missing.|[MyError](#schemamyerror)|
|401|[Unauthorized](https://tools.ietf.org/html/rfc7235#section-3.1)|# 401 Unauthorized
The authentication given was incorrect or insufficient.|[MyError](#schemamyerror)|
|default|Default|none|[CustomError](#schemacustomerror)|

<aside class="warning">
To perform this operation, you must be authenticated by means of one of the following methods:
ApiKeyAuth
</aside>

# Schemas

<h2 id="tocS_Map">Map</h2>
<!-- backwards compatibility -->
<a id="schemamap"></a>
<a id="schema_Map"></a>
<a id="tocSmap"></a>
<a id="tocsmap"></a>

```json
{
  "name": "string",
  "difficulty": "easy",
  "state": "new",
  "created_at": 0,
  "last_changed": 0
}

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|name|string|true|none|none|
|difficulty|[Difficulty](#schemadifficulty)|true|none|none|
|state|[MapState](#schemamapstate)|true|none|none|
|created_at|integer(uint64)|true|none|none|
|last_changed|integer(uint64)|true|none|none|

<h2 id="tocS_Difficulty">Difficulty</h2>
<!-- backwards compatibility -->
<a id="schemadifficulty"></a>
<a id="schema_Difficulty"></a>
<a id="tocSdifficulty"></a>
<a id="tocsdifficulty"></a>

```json
"easy"

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|none|

#### Enumerated Values

|Property|Value|
|---|---|
|*anonymous*|easy|
|*anonymous*|main|
|*anonymous*|hard|
|*anonymous*|insane|

<h2 id="tocS_MapState">MapState</h2>
<!-- backwards compatibility -->
<a id="schemamapstate"></a>
<a id="schema_MapState"></a>
<a id="tocSmapstate"></a>
<a id="tocsmapstate"></a>

```json
"new"

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|*anonymous*|string|false|none|none|

#### Enumerated Values

|Property|Value|
|---|---|
|*anonymous*|new|
|*anonymous*|declined|
|*anonymous*|approved|
|*anonymous*|published|

<h2 id="tocS_MyError">MyError</h2>
<!-- backwards compatibility -->
<a id="schemamyerror"></a>
<a id="schema_MyError"></a>
<a id="tocSmyerror"></a>
<a id="tocsmyerror"></a>

```json
{
  "err": "string",
  "msg": "string"
}

```

Error messages returned to user

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|err|string|true|none|The title of the error message|
|msg|string¦null|false|none|The description of the error|

<h2 id="tocS_CustomError">CustomError</h2>
<!-- backwards compatibility -->
<a id="schemacustomerror"></a>
<a id="schema_CustomError"></a>
<a id="tocScustomerror"></a>
<a id="tocscustomerror"></a>

```json
{
  "msg": "string",
  "code": 0
}

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|msg|string|true|none|none|
|code|integer(uint16)|true|none|none|

<h2 id="tocS_CreateMapData">CreateMapData</h2>
<!-- backwards compatibility -->
<a id="schemacreatemapdata"></a>
<a id="schema_CreateMapData"></a>
<a id="tocScreatemapdata"></a>
<a id="tocscreatemapdata"></a>

```json
{
  "name": "string",
  "difficulty": "string",
  "url": "string"
}

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|name|string|true|none|none|
|difficulty|string|true|none|none|
|url|string|true|none|none|

<h2 id="tocS_ChangeMapDifficultyData">ChangeMapDifficultyData</h2>
<!-- backwards compatibility -->
<a id="schemachangemapdifficultydata"></a>
<a id="schema_ChangeMapDifficultyData"></a>
<a id="tocSchangemapdifficultydata"></a>
<a id="tocschangemapdifficultydata"></a>

```json
{
  "name": "string",
  "difficulty": "string"
}

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|name|string|true|none|none|
|difficulty|string|true|none|none|

<h2 id="tocS_JustTheMapName">JustTheMapName</h2>
<!-- backwards compatibility -->
<a id="schemajustthemapname"></a>
<a id="schema_JustTheMapName"></a>
<a id="tocSjustthemapname"></a>
<a id="tocsjustthemapname"></a>

```json
{
  "name": "string"
}

```

### Properties

|Name|Type|Required|Restrictions|Description|
|---|---|---|---|---|
|name|string|true|none|none|

