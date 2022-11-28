use serde::{Deserialize, Serialize};

// {
//   "personalizations": [
//     {
//       "to": [
//         {
//           "email": "to@email.com"
//         }
//       ]
//     }
//   ],
//   "from": {
//     "email": "from@email.com"
//   },
//   "subject": "Subject line",
//   "content": [
//     {
//       "type": "text/plain",
//       "value": "Content line"
//     }
//   ]
// }

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendgridEmailFormat<'a> {
    pub personalizations: Vec<PersonalizationField<'a>>,
    pub from: FromField<'a>,
    pub subject: &'a str,
    pub content: Vec<ContentField<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizationField<'a> {
    // https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl
    #[serde(borrow = "'a")]
    pub to: Vec<ToField<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToField<'a> {
    pub email: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FromField<'a> {
    pub email: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentField<'a> {
    #[serde(rename = "type")]
    pub type_field: &'a str,
    pub value: &'a str,
}
