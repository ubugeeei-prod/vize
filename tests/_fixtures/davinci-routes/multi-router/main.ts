import { createApp } from "vue";
import AdminApp from "./AdminApp.vue";
import SiteApp from "./SiteApp.vue";
import adminRouter from "./admin-router";
import { siteRouter } from "./site-router";
createApp(AdminApp).use(adminRouter);
createApp(SiteApp).use(siteRouter);
