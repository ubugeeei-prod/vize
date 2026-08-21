<MyPane :title.sync="pane.title" :width.sync="w" :other="x"></MyPane>
<Widget :model-name.sync></Widget>
<Row :[k].sync="v"></Row>
<button @click.native.stop="go" @keyup.13="submit" @keydown.99.native="odd"></button>
<Card><template slot="header" slot-scope="props"><b>{{ props.title }}</b></template></Card>
<List><template scope="row">item</template></List>
<Grid><template slot-scope="cell" #conflict="c">y</template></Grid>
<Tab><template slot="named">z</template></Tab>
