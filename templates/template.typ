#import sys: inputs

#set page(paper: "a4")
#set text(font: "Roboto", 12pt, fallback: false)

#let content = inputs.v
#let assigner = inputs.assigner.name
#let assignee = inputs.assignee.name
#let asset = inputs.asset.name
#let cc = inputs.at("cc", default: none)

#text(weight: "bold")[#align(center)[Contract on the use of provided data]]

#v(5%)

#strong[Contract on the use of provided data]
#linebreak()
between


#v(5%)
#strong[Data provider:]
#linebreak()
#assigner

#v(5%)
and
#v(5%)
#strong[Data user:] 
#linebreak()
#assignee

#v(10%)

For the access and use of the data asset: #strong[#asset]

#v(5%)

= Terms

#set par(justify: true)
#v(1%)

#if cc != none [
  #eval(cc, mode: "markup")
]else[
  #for (i, elem) in content.enumerate() [
  #par(justify: true)[
    == #i. #elem.heading
    #v(2%)
    #elem.text
    #lorem(50)
    ]
  ]
]