


## Typographic replacements

Enable typographer option to see result.

(c) (C) (r) (R) (tm) (TM) (p) (P) +-

test.. test... test..... test?..... test!....

!!!!!! ???? ,,  -- ---

"Smartypants, double quotes" and 'single quotes'


## Emphasis

**This is bold text**

__This is bold text__

*This is italic text*

_This is italic text_

~~Strikethrough~~


## Blockquotes


> Blockquotes can also be nested...
>> ...by using additional greater-than signs right next to each other...
> > > ...or with spaces between arrows.


## Lists

Unordered

+ Create a list by starting a line with `+`, `-`, or `*`
+ Sub-lists are made by indenting 2 spaces:
  - Marker character change forces new list start:
    * Ac tristique libero volutpat at
    + Facilisis in pretium nisl aliquet
    - Nulla volutpat aliquam velit
+ Very easy!

Ordered

1. Lorem ipsum dolor sit amet
2. Consectetur adipiscing elit
3. Integer molestie lorem at massa


1. You can use sequential numbers...
1. ...or keep all the numbers as `1.`

Start numbering with offset:

57. foo
1. bar


## Code

Inline `code`

Indented code

    // Some comments
    line 1 of code
    line 2 of code
    line 3 of code


Block code "fences"

```
Sample text here...
```

Syntax highlighting

``` js
var foo = function (bar) {
  return bar++;
};

console.log(foo(5));
```

## Tables

| Option | Description |
| ------ | ----------- |
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |

Right aligned columns

| Option | Description |
| ------:| -----------:|
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |


## Links

[link text](http://dev.nodeca.com)

[link with title](http://nodeca.github.io/pica/demo/ "title text!")

Autoconverted link https://github.com/nodeca/pica (enable linkify to see)


## Images

![Minion](https://octodex.github.com/images/minion.png)
![Stormtroopocat](https://octodex.github.com/images/stormtroopocat.jpg "The Stormtroopocat")

Like links, Images also have a footnote style syntax

![Alt text][id]

With a reference later in the document defining the URL location:

[id]: https://octodex.github.com/images/dojocat.jpg  "The Dojocat"

## System Design Diagrams
Diagrams are based on the diagrams in the [System Design Primer](https://github.com/donnemartin/system-design-primer/tree/master/solutions/system_design)

### [Mint](https://github.com/donnemartin/system-design-primer/tree/master/solutions/system_design/mint)

```mermaid
---
title: Mint (Simple)
---
graph TB
Client --> web[Web Server] --> accounts[Accounts API]
accounts --> tes[Transaction Extraction Service] --> category[Category Service] & budget[Budget Service] & notif[Notification Service]
accounts --> db
tes --> db[SQL]
tes --> store[Object Store]
```

```mermaid
---
title: Mint (Scaled)
---
graph TB
Client --> DNS & CDN & lb[Load Balancer]
lb --> web[Web Server]
subgraph api
web --> accounts[Accounts API] & read[Read API]
memoryCache[Memory Cache]
end

accounts --> queue[Queue] --> tes


subgraph storage
dbPrimary[(SQL Write Primary)] -.- dbReplica[(SQL Read Replicas)]
objectStore[(Object Store)]
end

subgraph services
tes[Transaction Extraction Service] --> category[Category Service] & budget[Budget Service] & notif[Notification Service]
end

tes --> objectStore
CDN --> objectStore
tes --> dbPrimary
accounts --> dbPrimary & dbReplica
read --> dbReplica & memoryCache[Memory Cache]
```

### [Sales Rank](https://github.com/donnemartin/system-design-primer/tree/master/solutions/system_design/sales_rank)

```mermaid
---
title: Sales Rank (Simple)
---
graph TD
Client --> web[Web Server]
web --> salesApi[Sales API] & readApi[Read API]
service[Sales Rank Service]

subgraph storage
db[SQL]
store[Object Store]
end

service --> db & store
salesApi --> db & store
readApi --> db
```

```mermaid
---
title: Sales Rank (Scaled)
---
graph TD
Client --> DNS & CDN & lb[Load Balancer]
CDN --> store
lb --> web[Web Server] --> salesApi[Sales API] & readApi[Read API]
service[Sales Rank Service]

subgraph storage
dbPrimary[SQL Write Primary] -.- dbReplica[SQL Read Replicas]
store[Object Store]
end

service --> dbPrimary & store
salesApi --> dbPrimary & store
readApi --> dbReplica & store & memoryCache[Memory Cache]

subgraph apis
salesApi
readApi
end

subgraph frontend
Client
DNS
CDN
lb
web
end
```
