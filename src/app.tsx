import "./main.css";
import { Route, Switch } from "wouter";
import { Home } from "./pages/home";
import { Course } from "./pages/course";
import { GlobalLayout } from "./components/layout/global-layout";
import { Announcements } from "./pages/announcements";
import { Index } from "./pages";

export function App() {
	return (
		<Switch>
			<Route path="/">
				<Index />
			</Route>
			<GlobalLayout>
				<Route path="/home">
					<Home />
				</Route>
				<Route path="/announcements">
					<Announcements />
				</Route>
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
