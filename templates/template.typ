#import sys: inputs

#set page(paper: "a4")
#set text(font: "Roboto", 12pt, fallback: false)

#let content = inputs.v
#let assigner = inputs.assigner
#let assignee = inputs.assignee
#let asset = inputs.asset
#let odrl = bytes(inputs.odrl)
#let counter = counter("c")

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

#counter.step()

#set par(justify: true)
#v(1%)

#for (i, elem) in content.enumerate() [
  #par(justify: true)[
    #if elem.heading.len() > 0 [
      == #context counter.display(). #elem.heading
      #v(2%)
      #counter.step()
    ]
    #if elem.text.len() > 0 [
      #eval(elem.text, mode: "markup")
      #parbreak()
    ]
  ]
]

#pdf.embed.decode(odrl, "odrl.jsonld", name: "odrl.jsonld", description: "Associated ODRL policy", mime-type: "application/ld+json", relationship: "supplement")