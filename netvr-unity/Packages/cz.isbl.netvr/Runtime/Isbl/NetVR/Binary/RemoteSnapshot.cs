using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class RemoteSnapshot: IEquatable<RemoteSnapshot>, ICloneable {
        public Serde.ValueDictionary<uint, RemoteClientSnapshot> clients;

        public RemoteSnapshot(Serde.ValueDictionary<uint, RemoteClientSnapshot> _clients) {
            if (_clients == null) throw new ArgumentNullException(nameof(_clients));
            clients = _clients;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            TraitHelpers.serialize_map_u32_to_RemoteClientSnapshot(clients, serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static RemoteSnapshot Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RemoteSnapshot obj = new RemoteSnapshot(
            	TraitHelpers.deserialize_map_u32_to_RemoteClientSnapshot(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RemoteSnapshot BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RemoteSnapshot BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RemoteSnapshot value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RemoteSnapshot other && Equals(other);

        public static bool operator ==(RemoteSnapshot left, RemoteSnapshot right) => Equals(left, right);

        public static bool operator !=(RemoteSnapshot left, RemoteSnapshot right) => !Equals(left, right);

        public bool Equals(RemoteSnapshot other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!clients.Equals(other.clients)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + clients.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RemoteSnapshot Clone() => (RemoteSnapshot)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
