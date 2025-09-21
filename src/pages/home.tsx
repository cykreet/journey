import IconJourney from "~icons/journey/journey?color=red";

export function Home() {
	return (
		// todo: could move page containers to separate component for consistent top margins etc
		<div className="flex flex-col justify-center items-center mt-10">
			<IconJourney className="w-14 h-14 text-steel-300" />
		</div>
	);
}
