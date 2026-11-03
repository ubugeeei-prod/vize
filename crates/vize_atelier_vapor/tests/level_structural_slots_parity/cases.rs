//! Reviewed official rc.9 workload: branches, renames, reordering and teardown.
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub(super) struct Case {
    pub(super) name: String,
    pub(super) source: String,
    pub(super) child: String,
    pub(super) context: Value,
    pub(super) steps: Value,
}

pub(super) fn cases() -> Vec<Case> {
    serde_json::from_str(r#"[
  {
    "source": "<main data-id=\"root\"><Child :value=\"childValue\"><template #one=\"p\" v-if=\"enabled\"><button data-id=\"content\" @click=\"record(label+':'+p.value)\">{{label}}:{{p.value}}</button></template><template #[name]=\"p\" v-else-if=\"second\"><button data-id=\"alternate\" @click=\"record(p.value)\">{{label}}:{{p.value}}</button></template></Child><i data-id=\"tail\">tail</i></main>",
    "child": "<section data-id=\"child\"><slot name=\"one\" :value=\"value\"><em data-id=\"one-fallback\">one</em></slot><slot name=\"two\" :value=\"value\"><em data-id=\"two-fallback\">two</em></slot></section>",
    "context": {
      "enabled": true,
      "second": false,
      "name": "two",
      "label": "A",
      "childValue": "P0"
    },
    "steps": [
      {
        "patch": {
          "label": "B",
          "childValue": "P1"
        }
      },
      {
        "click": "content"
      },
      {
        "patch": {
          "enabled": false,
          "second": true
        }
      },
      {
        "patch": {
          "name": "one"
        }
      },
      {
        "click": "alternate"
      },
      {
        "patch": {
          "second": false
        }
      },
      {
        "patch": {
          "enabled": true,
          "label": "C"
        }
      }
    ],
    "name": "conditional"
  },
  {
    "source": "<main data-id=\"root\"><Child :value=\"childValue\"><template v-for=\"(item, key) in items\" #[item.name]=\"p\"><button :data-id=\"item.id\" @click=\"record(item.label+':'+key+':'+p.value)\">{{item.label}}:{{key}}:{{p.value}}</button></template></Child><i data-id=\"tail\">tail</i></main>",
    "child": "<section data-id=\"child\"><slot name=\"one\" :value=\"value\"><em data-id=\"one-fallback\">one</em></slot><slot name=\"two\" :value=\"value\"><em data-id=\"two-fallback\">two</em></slot></section>",
    "context": {
      "items": [
        {
          "id": "a",
          "name": "one",
          "label": "A"
        },
        {
          "id": "b",
          "name": "two",
          "label": "B"
        }
      ],
      "childValue": "P0"
    },
    "steps": [
      {
        "patch": {
          "items": [
            {
              "id": "b",
              "name": "two",
              "label": "B2"
            },
            {
              "id": "a",
              "name": "one",
              "label": "A2"
            }
          ],
          "childValue": "P1"
        }
      },
      {
        "click": "a"
      },
      {
        "patch": {
          "items": [
            {
              "id": "b",
              "name": "two",
              "label": "B3"
            },
            {
              "id": "c",
              "name": "one",
              "label": "C"
            }
          ]
        }
      },
      {
        "click": "c"
      },
      {
        "patch": {
          "items": [
            {
              "id": "b",
              "name": "one",
              "label": "B4"
            }
          ]
        }
      },
      {
        "patch": {
          "items": []
        }
      },
      {
        "patch": {
          "items": [
            {
              "id": "a",
              "name": "two",
              "label": "A3"
            }
          ]
        }
      }
    ],
    "name": "loop"
  },
  {
    "source": "<main data-id=\"root\"><Child :value=\"childValue\"><template v-for=\"(item, key, index) in items\" #[item.name]=\"{ value }\"><button :data-id=\"key\" @click=\"record(item.label+':'+key+':'+index+':'+value)\">{{item.label}}:{{key}}:{{index}}:{{value}}</button></template></Child><i data-id=\"tail\">tail</i></main>",
    "child": "<section data-id=\"child\"><slot name=\"one\" :value=\"value\"><em data-id=\"one-fallback\">one</em></slot><slot name=\"two\" :value=\"value\"><em data-id=\"two-fallback\">two</em></slot></section>",
    "context": {
      "items": {
        "a": {
          "name": "one",
          "label": "A"
        },
        "b": {
          "name": "two",
          "label": "B"
        }
      },
      "childValue": "P0"
    },
    "steps": [
      {
        "patch": {
          "items": {
            "b": {
              "name": "two",
              "label": "B2"
            },
            "a": {
              "name": "one",
              "label": "A2"
            }
          },
          "childValue": "P1"
        }
      },
      {
        "click": "a"
      },
      {
        "patch": {
          "items": {
            "c": {
              "name": "one",
              "label": "C"
            },
            "b": {
              "name": "two",
              "label": "B3"
            }
          }
        }
      },
      {
        "click": "c"
      },
      {
        "patch": {
          "items": {}
        }
      }
    ],
    "name": "object"
  }
]"#).unwrap()
}
