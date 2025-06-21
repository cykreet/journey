import "./main.css";
import { Route, Switch } from "wouter";
import { Index } from "./views";
import { Course } from "./views/course";
import { GlobalLayout } from "./components/layout/global-layout";
import { MenuLayout } from "./components/layout/menu/menu-layout";
import { Announcements } from "./views/announcements";
import type { MenuSidebarItem } from "./components/layout/menu/menu-sidebar";
import IconSpeaker from "~icons/tabler/device-speaker-filled";

export function App() {
	const homeMenuItems: MenuSidebarItem[] = [
		{
			id: 1,
			icon: IconSpeaker,
			name: "Announcements",
			href: "/announcements",
		},
	];

	return (
		<Switch>
			<GlobalLayout>
				<MenuLayout header={"Journey"} sidebarItems={homeMenuItems}>
					<Route path="/">
						<Index />
					</Route>
					<Route path="/announcements">
						<Announcements />
					</Route>
				</MenuLayout>
				<Route path="/course/:id?/:page?">
					<Course />
				</Route>
			</GlobalLayout>
			<Route>
				<p>404 not found</p>
			</Route>
		</Switch>
	);
}

export default App;
