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
pub struct SendgridEmailFormat {
    pub personalizations: Vec<PersonalizationField>,
    pub from: FromField,
    pub subject: String,
    pub content: Vec<ContentField>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizationField {
    pub to: Vec<ToField>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToField {
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FromField {
    pub email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentField {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: String,
}
