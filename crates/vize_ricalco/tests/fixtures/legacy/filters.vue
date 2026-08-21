<div>{{ message | capitalize }}</div>
<p>{{ a | f(b) | g }}</p>
<span :id="raw | formatId" :title="'a|b' | quote"></span>
<i>{{ a || b }}</i>
<b>{{ x | (bad) }}</b>
<u>{{ z | h | capitalize }}</u>
<em>pre {{ m | f }}</em>
