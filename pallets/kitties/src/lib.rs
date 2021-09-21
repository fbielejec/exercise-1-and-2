#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

    use codec::{Decode, Encode};
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use pallet_randomness_collective_flip;
    use sp_api::HashT;

    #[derive(Encode, Decode, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Kitty<Hash> {
        id: Hash,
        dna: Hash,
    }

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_randomness_collective_flip::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // STORAGE
    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage

    #[pallet::storage]
    #[pallet::getter(fn get_nonce)]
    pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    // a map from a kitten id to user id
    #[pallet::storage]
    #[pallet::getter(fn get_kittens)]
    pub(super) type Kittens<T: Config> = StorageMap<
        _,                // prefix
        Blake2_128Concat, // hasher
        T::Hash,          // key
        T::AccountId,     // value
        ValueQuery,
    >;

    // EVENTS
    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [kittenId, ownerId]
        KittenCreated(T::Hash, T::AccountId),
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn create_kitten(origin: OriginFor<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            let nonce = <Nonce<T>>::get();
            // seed dependent on previous 80 blocks
            let seed = <pallet_randomness_collective_flip::Pallet<T>>::random_material();
            // Generate a random hash, using the nonce and sender account id as part of the hash
            let random_hash = T::Hashing::hash_of(&(seed, &sender, nonce));

            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
            };

            // Update storage
            <Nonce<T>>::mutate(|n| *n += 1);
            <Kittens<T>>::insert(new_kitty.id, &sender);

            // Emit an event
            Self::deposit_event(Event::KittenCreated(new_kitty.id, sender));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
    }
}
