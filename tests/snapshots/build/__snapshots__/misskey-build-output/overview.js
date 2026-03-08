import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { markRaw, onMounted, onBeforeUnmount, nextTick, shallowRef, ref, computed, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import XFederation from './overview.federation.vue'
import XInstances from './overview.instances.vue'
import XQueue from './overview.queue.vue'
import XApRequests from './overview.ap-requests.vue'
import XUsers from './overview.users.vue'
import XActiveUsers from './overview.active-users.vue'
import XStats from './overview.stats.vue'
import XRetention from './overview.retention.vue'
import XModerators from './overview.moderators.vue'
import XHeatmap from './overview.heatmap.vue'
import type { InstanceForPie } from './overview.pie.vue'
import * as os from '@/os.js'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview',
  setup(__props) {

const rootEl = useTemplateRef('rootEl');
const serverInfo = ref<Misskey.entities.ServerInfoResponse | null>(null);
const topSubInstancesForPie = ref<InstanceForPie[] | null>(null);
const topPubInstancesForPie = ref<InstanceForPie[] | null>(null);
const federationPubActive = ref<number | null>(null);
const federationPubActiveDiff = ref<number | null>(null);
const federationSubActive = ref<number | null>(null);
const federationSubActiveDiff = ref<number | null>(null);
const newUsers = ref<Misskey.entities.UserDetailed[] | null>(null);
const activeInstances = shallowRef<Misskey.entities.FederationInstancesResponse | null>(null);
const queueStatsConnection = markRaw(useStream().useChannel('queueStats'));
const now = new Date();
const filesPagination = {
	endpoint: 'admin/drive/files' as const,
	limit: 9,
	noPaging: true,
};
function onInstanceClick(i: Misskey.entities.FederationInstance) {
	os.pageWindow(`/instance-info/${i.host}`);
}
onMounted(async () => {
	/*
	const magicGrid = new MagicGrid({
		container: rootEl,
		static: true,
		animate: true,
	});
	magicGrid.listen();
	*/
	misskeyApiGet('charts/federation', { limit: 2, span: 'day' }).then(chart => {
		federationPubActive.value = chart.pubActive[0];
		federationPubActiveDiff.value = chart.pubActive[0] - chart.pubActive[1];
		federationSubActive.value = chart.subActive[0];
		federationSubActiveDiff.value = chart.subActive[0] - chart.subActive[1];
	});
	misskeyApiGet('federation/stats', { limit: 10 }).then(res => {
		topSubInstancesForPie.value = [
			...res.topSubInstances.map(x => ({
				name: x.host,
				color: x.themeColor,
				value: x.followersCount,
				onClick: () => {
					os.pageWindow(`/instance-info/${x.host}`);
				},
			})),
			{ name: '(other)', color: '#80808080', value: res.otherFollowersCount },
		];
		topPubInstancesForPie.value = [
			...res.topPubInstances.map(x => ({
				name: x.host,
				color: x.themeColor,
				value: x.followingCount,
				onClick: () => {
					os.pageWindow(`/instance-info/${x.host}`);
				},
			})),
			{ name: '(other)', color: '#80808080', value: res.otherFollowingCount },
		];
	});
	misskeyApi('admin/server-info').then(serverInfoResponse => {
		serverInfo.value = serverInfoResponse;
	});
	misskeyApi('admin/show-users', {
		limit: 5,
		sort: '+createdAt',
	}).then(res => {
		newUsers.value = res;
	});
	misskeyApi('federation/instances', {
		sort: '+latestRequestReceivedAt',
		limit: 25,
	}).then(res => {
		activeInstances.value = res;
	});
	nextTick(() => {
		queueStatsConnection.send('requestLog', {
			id: genId(),
			length: 100,
		});
	});
});
onBeforeUnmount(() => {
	queueStatsConnection.dispose();
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.dashboard,
	icon: 'ti ti-dashboard',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 1000px;"
    }, [ _createElementVNode("div", {
        ref_key: "rootEl", ref: rootEl,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Stats")
          ]),
          default: _withCtx(() => [
            _createVNode(XStats)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Active users")
          ]),
          default: _withCtx(() => [
            _createVNode(XActiveUsers)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Heatmap")
          ]),
          default: _withCtx(() => [
            _createVNode(XHeatmap)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Retention rate")
          ]),
          default: _withCtx(() => [
            _createVNode(XRetention)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Moderators")
          ]),
          default: _withCtx(() => [
            _createVNode(XModerators)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Federation")
          ]),
          default: _withCtx(() => [
            _createVNode(XFederation)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Instances")
          ]),
          default: _withCtx(() => [
            _createVNode(XInstances)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Ap requests")
          ]),
          default: _withCtx(() => [
            _createVNode(XApRequests)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("New users")
          ]),
          default: _withCtx(() => [
            _createVNode(XUsers)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Deliver queue")
          ]),
          default: _withCtx(() => [
            _createVNode(XQueue, { domain: "deliver" })
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkFoldableSection, { class: "item" }, {
          header: _withCtx(() => [
            _createTextVNode("Inbox queue")
          ]),
          default: _withCtx(() => [
            _createVNode(XQueue, { domain: "inbox" })
          ]),
          _: 1 /* STABLE */
        }) ], 512 /* NEED_PATCH */) ]))
}
}

})
